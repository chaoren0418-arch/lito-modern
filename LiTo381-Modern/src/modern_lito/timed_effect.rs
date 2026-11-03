//! Shared-only one-shot presentation for timed item effects 191, 750 and 751.
//!
//! The inspected shared Text/list.spr (SHA256 523c61ad...b9f127) defines
//! undirected action0 at 24 FPS, attr104=8. Effect191 is the ten-source-frame
//! potion sequence, with no events. Effects750/751 have six source frames,
//! each lasting five ticks; source1 has native `[142` / `[121` respectively.
//! Native one-shot playback starts at source1 and returns to terminal source0
//! without dwelling there: 15 ticks for191, 25 ticks for750/751. The extra five
//! decoded SPR subframes in750/751 are retained/validated but never invented as
//! additional action frames. No Hit, actor override, buff timer or gameplay
//! action is generated here. The caller owns authoritative event deduplication.
//!
//! Worker-only all-or-nothing loading: bounded indexes/encoded data/decoded RGBA
//! and qualified shared WAVs, cooperative cancellation between reads/decode.
//! No file handle survives loading. Rendering and playback sampling are memory
//! only. Hash-selected SpriteNN is used first; only missing membership permits
//! base Sprite fallback. Corruption/ambiguity, LT namespaces, unknown controls,
//! replacement images/audio and PCM rate changes are never fallback paths.

pub use super::teleport_effect::{SpriteProvenance, VisualSample};
use super::{
    combat_animation::{classify_native_cues, CrossedCue, NativeAudioGroup},
    combat_audio::{BankLimits, SoundBank},
    compositor::Canvas,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    shared_action_catalog::{SharedActionCatalog, SharedCatalogError},
    sprite::{
        decode_classic_spr_with_limits, DecodedSprite, SpriteError, SpriteFrame, SpriteLimits,
    },
    sprite_actions::{ActionError, ActionKey, Directionality, FrameEvent, SpriteActionCatalog},
    teleport_effect::{draw_screen_frame, TeleportError},
    world_map::PixelPoint,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

pub const EFFECTS: [i32; 3] = [191, 750, 751];
#[cfg(test)]
#[path = "timed_effect_reference_tests.rs"]
mod reference_tests;
const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 24 * 1024 * 1024;
const MAX_RGBA: usize = 32 * 1024 * 1024;
const MAX_INDEX_RECORDS: usize = 1_000_000;
const NANOS: u128 = 1_000_000_000;
const POTION_SUBFRAMES: [usize; 10] = [5, 0, 1, 2, 3, 4, 3, 4, 3, 4];
const POTION_DURATIONS: [u32; 10] = [3, 1, 1, 1, 2, 2, 2, 2, 2, 2];
const POTION_ENDS: [u128; 9] = [1, 2, 3, 5, 7, 9, 11, 13, 15];
const POWER_SUBFRAMES: [usize; 6] = [5, 0, 1, 2, 3, 4];
const POWER_DURATIONS: [u32; 6] = [5; 6];
const POWER_ENDS: [u128; 5] = [5, 10, 15, 20, 25];

#[derive(Debug)]
pub enum TimedError {
    Cancelled,
    InvalidInput(&'static str),
    Unsupported(&'static str),
    UnsupportedSoundBank,
    Bound(&'static str),
    Resource(ResourceError),
    Catalog(ActionError),
    Sprite(SpriteError),
    Render(TeleportError),
}
impl fmt::Display for TimedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "timed item resource loading cancelled"),
            Self::InvalidInput(reason) => write!(f, "invalid timed item presentation: {reason}"),
            Self::Unsupported(reason) => {
                write!(f, "unsupported shared timed item presentation: {reason}")
            }
            Self::UnsupportedSoundBank => {
                write!(f, "shared timed item sound bank 121/142 is unqualified")
            }
            Self::Bound(reason) => write!(f, "timed item presentation exceeds {reason} bound"),
            Self::Resource(error) => write!(f, "timed item shared resource: {error}"),
            Self::Catalog(error) => write!(f, "timed item action definition: {error}"),
            Self::Sprite(error) => write!(f, "timed item SPR: {error}"),
            Self::Render(error) => write!(f, "timed item drawing: {error}"),
        }
    }
}
impl std::error::Error for TimedError {}
impl From<SharedCatalogError> for TimedError {
    fn from(error: SharedCatalogError) -> Self {
        match error {
            SharedCatalogError::Cancelled => Self::Cancelled,
            SharedCatalogError::Bound(reason) => Self::Bound(reason),
            SharedCatalogError::Resource(error) => Self::Resource(error),
            SharedCatalogError::Action(error) => Self::Catalog(error),
        }
    }
}
impl From<ResourceError> for TimedError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
impl From<ActionError> for TimedError {
    fn from(value: ActionError) -> Self {
        Self::Catalog(value)
    }
}
impl From<SpriteError> for TimedError {
    fn from(value: SpriteError) -> Self {
        Self::Sprite(value)
    }
}
type Result<T> = std::result::Result<T, TimedError>;

#[derive(Clone, Copy)]
struct Definition {
    subframes: &'static [usize],
    durations: &'static [u32],
    ends: &'static [u128],
    decoded_frames: usize,
    sound: Option<u32>,
}
fn definition(effect: i32) -> Result<Definition> {
    match effect {
        191 => Ok(Definition {
            subframes: &POTION_SUBFRAMES,
            durations: &POTION_DURATIONS,
            ends: &POTION_ENDS,
            decoded_frames: 6,
            sound: None,
        }),
        750 | 751 => Ok(Definition {
            subframes: &POWER_SUBFRAMES,
            durations: &POWER_DURATIONS,
            ends: &POWER_ENDS,
            decoded_frames: 11,
            sound: Some(if effect == 750 { 142 } else { 121 }),
        }),
        _ => Err(TimedError::Unsupported("effect ID is not 191/750/751")),
    }
}
fn active(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(TimedError::Cancelled)
    } else {
        Ok(())
    }
}
fn sound_group(def: Definition) -> Result<Option<NativeAudioGroup>> {
    let Some(id) = def.sound else {
        return Ok(None);
    };
    let classified = classify_native_cues(&[FrameEvent::Sound(id)])
        .map_err(|_| TimedError::Unsupported("native timed sound classification"))?;
    if !classified.non_audio.is_empty() || classified.audio.is_none() {
        return Err(TimedError::Unsupported("native timed sound classification"));
    }
    Ok(classified.audio)
}
fn qualify(catalog: &SpriteActionCatalog, effect: i32) -> Result<Option<NativeAudioGroup>> {
    let def = definition(effect)?;
    let id = effect as u32;
    let sprite = catalog
        .sprites
        .get(&id)
        .ok_or(TimedError::Unsupported("missing timed item definition"))?;
    let resolved = catalog.resolve_action(id, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != id
        || sprite.image_count != Some(1)
        || resolved.definition_sprite_id != id
        || sprite.actions.len() != 1
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != def.subframes.len()
        || sprite.attributes.len() != 1
        || sprite.attributes[0].key != ActionKey::new(104)
        || sprite.attributes[0].body_bytes != b"8"
    {
        return Err(TimedError::Unsupported(
            "unverified timed item controls or graphics",
        ));
    }
    for (index, frame) in animation.frames.iter().enumerate() {
        let expected_events: Vec<FrameEvent> = if index == 1 {
            def.sound.map(FrameEvent::Sound).into_iter().collect()
        } else {
            Vec::new()
        };
        if frame.image_id != 0
            || frame.subframe as usize != def.subframes[index]
            || frame.duration_ticks != def.durations[index]
            || frame.events != expected_events
        {
            return Err(TimedError::Unsupported(
                "unverified timed item frames or cues",
            ));
        }
        for heading in 0..8 {
            let asset = animation.frame_asset(id, index, heading)?;
            if asset.graphics_id != id
                || asset.image_id != 0
                || asset.subframe as usize != def.subframes[index]
            {
                return Err(TimedError::Unsupported("unverified timed item SPR route"));
            }
        }
    }
    sound_group(def)
}
fn sample(def: Definition, elapsed: Duration) -> Result<(usize, bool)> {
    let tick = elapsed
        .as_nanos()
        .checked_mul(24)
        .ok_or(TimedError::Bound("elapsed time"))?
        / NANOS;
    if tick
        >= *def
            .ends
            .last()
            .ok_or(TimedError::Unsupported("empty timeline"))?
    {
        Ok((0, true))
    } else {
        Ok((def.ends.partition_point(|end| *end <= tick) + 1, false))
    }
}
fn duration(def: Definition) -> Duration {
    Duration::from_nanos(
        ((*def.ends.last().expect("fixed nonempty definition") * NANOS).div_ceil(24)) as u64,
    )
}

pub struct TimedSample {
    pub source_frame: usize,
    pub finished: bool,
    /// Exact native source1 `[id` marker, at tick0. Never a gameplay command.
    pub crossed_cues: Vec<CrossedCue>,
}
pub struct TimedPlayback {
    definition: Definition,
    last_elapsed: Option<Duration>,
}
impl TimedPlayback {
    /// Monotonic one-shot cursor. A newly observed event returns its first cue
    /// once even if the first sample is late; caller controls stale-event policy.
    /// Repeated samples, the terminal hold and Duration::MAX cannot replay it.
    pub fn advance(&mut self, elapsed: Duration) -> Result<TimedSample> {
        if self.last_elapsed.is_some_and(|last| elapsed < last) {
            return Err(TimedError::InvalidInput("elapsed time regressed"));
        }
        let (source_frame, finished) = sample(self.definition, elapsed)?;
        let crossed_cues = if self.last_elapsed.is_none() {
            self.definition
                .sound
                .map(|id| CrossedCue {
                    source_frame: 1,
                    event_index: 0,
                    tick: 0,
                    event: FrameEvent::Sound(id),
                })
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };
        self.last_elapsed = Some(elapsed);
        Ok(TimedSample {
            source_frame,
            finished,
            crossed_cues,
        })
    }
}

struct LoadedEffect {
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
        active(cancel)?;
        let records = text_records
            .checked_add(base.len())
            .ok_or(TimedError::Bound("index records"))?;
        if records > MAX_INDEX_RECORDS {
            return Err(TimedError::Bound("index records"));
        }
        Ok(Self {
            root: base.root().to_owned(),
            records,
            base,
            shards: BTreeMap::new(),
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
                .ok_or(TimedError::Bound("index records"))?;
            if self.records > MAX_INDEX_RECORDS {
                return Err(TimedError::Bound("index records"));
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
            Err(error) => Err(error.into()),
        }
    }
}

pub struct TimedAssets {
    effects: BTreeMap<i32, LoadedEffect>,
    sound: Arc<SoundBank>,
    groups: Vec<NativeAudioGroup>,
    catalog_sha256: [u8; 32],
    encoded: usize,
    rgba: usize,
}
impl TimedAssets {
    /// Worker-only loading of all3effects plus both unmodified shared WAVs.
    /// <=8MiB/SPR, <=24MiB SPR total, <=32MiB RGBA, <=2MiB sound bank;
    /// the action catalog is separately bounded to8MiB. Index total<=1million.
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        Self::load_with_catalog(&catalog, cancel)
    }

    pub(crate) fn load_with_catalog(
        context: &SharedActionCatalog,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let root = context.root();
        let records = context.index_records();
        let size = context.byte_length();
        if records > MAX_INDEX_RECORDS || size == 0 || size > MAX_ENTRY {
            return Err(TimedError::Bound("action catalog"));
        }
        active(cancel)?;
        let catalog_sha256 = context.sha256();
        let catalog = context.catalog();
        let mut groups = Vec::new();
        for effect in EFFECTS {
            active(cancel)?;
            if let Some(group) = qualify(&catalog, effect)? {
                groups.push(group);
            }
        }
        let mut reader = SpriteReader::open(root, records, cancel)?;
        let mut effects = BTreeMap::new();
        let mut encoded = 0usize;
        let mut rgba = 0usize;
        for effect in EFFECTS {
            active(cancel)?;
            let def = definition(effect)?;
            let name = format!("{effect}-0.spr");
            let archive = reader.archive_for(&name, cancel)?;
            let size = archive.entry(&name)?.size as usize;
            if size == 0 || size > MAX_ENTRY || size > MAX_ENCODED - encoded {
                return Err(TimedError::Bound("encoded sprites"));
            }
            let kind = archive.kind();
            active(cancel)?;
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            if bytes.len() != size {
                return Err(TimedError::Unsupported("SPR length changed"));
            }
            let remaining = (MAX_RGBA - rgba) / 4;
            if remaining == 0 {
                return Err(TimedError::Bound("decoded sprites"));
            }
            let decoded = decode_classic_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_frames: def.decoded_frames,
                    max_total_pixels: remaining,
                    ..Default::default()
                },
            )?;
            active(cancel)?;
            if decoded.frames.len() != def.decoded_frames {
                return Err(TimedError::Unsupported(
                    "missing or extra timed item SPR subframes",
                ));
            }
            let count = decoded
                .frames
                .iter()
                .try_fold(0usize, |sum, frame| sum.checked_add(frame.rgba.len()))
                .ok_or(TimedError::Bound("decoded sprites"))?;
            if count > MAX_RGBA - rgba {
                return Err(TimedError::Bound("decoded sprites"));
            }
            encoded += size;
            rgba += count;
            effects.insert(
                effect,
                LoadedEffect {
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
        active(cancel)?;
        let sound = SoundBank::load_shared(
            root,
            &groups,
            BankLimits {
                max_bytes: 2 * 1024 * 1024,
                max_sounds: 2,
                max_groups: 2,
            },
            cancel,
        )
        .map_err(|_| {
            if cancel.load(Ordering::Acquire) {
                TimedError::Cancelled
            } else {
                TimedError::UnsupportedSoundBank
            }
        })?;
        active(cancel)?;
        Ok(Self {
            effects,
            sound: Arc::new(sound),
            groups,
            catalog_sha256,
            encoded,
            rgba,
        })
    }
    fn effect(&self, effect: i32) -> Result<&LoadedEffect> {
        definition(effect)?;
        self.effects
            .get(&effect)
            .ok_or(TimedError::Unsupported("effect was not preloaded"))
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.catalog_sha256
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.effects.values().map(|effect| &effect.provenance)
    }
    /// Read-only access to every decoded SPR frame, including non-action frames.
    /// Useful for independent resource provenance/golden checks, not playback.
    pub fn sprite_frames(&self, effect: i32) -> Result<&[SpriteFrame]> {
        Ok(&self.effect(effect)?.decoded.frames)
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
    pub fn sound_groups(&self) -> &[NativeAudioGroup] {
        &self.groups
    }
    pub fn sound_group(&self, effect: i32) -> Result<Option<&NativeAudioGroup>> {
        self.effect(effect)?;
        let Some(id) = definition(effect)?.sound else {
            return Ok(None);
        };
        self.groups
            .iter()
            .find(|group| group.candidates() == [id])
            .map(Some)
            .ok_or(TimedError::Unsupported("missing qualified sound group"))
    }
    pub fn duration(&self, effect: i32) -> Result<Duration> {
        self.effect(effect)?;
        Ok(duration(definition(effect)?))
    }
    pub fn start_effect(&self, effect: i32) -> Result<TimedPlayback> {
        self.effect(effect)?;
        Ok(TimedPlayback {
            definition: definition(effect)?,
            last_elapsed: None,
        })
    }
    pub fn effect_frame(&self, effect: i32, elapsed: Duration) -> Result<VisualSample<'_>> {
        let loaded = self.effect(effect)?;
        let def = definition(effect)?;
        let (source_frame, finished) = sample(def, elapsed)?;
        let subframe = def.subframes[source_frame];
        let frame = loaded
            .decoded
            .frames
            .get(subframe)
            .ok_or(TimedError::Unsupported("unloaded timed item subframe"))?;
        Ok(VisualSample {
            frame,
            source_frame,
            finished,
            graphics_id: effect as u32,
            sprite_name: &loaded.provenance.name,
            subframe,
        })
    }
    /// The caller supplies the confirmed use event's projected origin. Motion
    /// after that use does not relocate this overlay. Terminal0 is inspectable
    /// through effect_frame but never held on the canvas after completion.
    pub fn draw_effect(
        &self,
        canvas: &mut Canvas,
        effect: i32,
        anchor: PixelPoint,
        elapsed: Duration,
    ) -> Result<bool> {
        let sample = self.effect_frame(effect, elapsed)?;
        if sample.finished {
            return Ok(false);
        }
        draw_screen_frame(canvas, sample.frame, anchor).map_err(TimedError::Render)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{combat_animation::NativeAudioMode, sprite_actions::ActionDefinition};
    use super::*;

    fn catalog() -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(b"3 0 0\n#191 1 green potion\n0.fire(0 10,0.5:3 0.0:1 0.1:1 0.2:1 0.3:2 0.4:2 0.3:2 0.4:2 0.3:2 0.4:2) 104.attr(8)\n#750 1 strength\n0.fire(0 6,0.5:5 0.0:5[142 0.1:5 0.2:5 0.3:5 0.4:5) 104.attr(8)\n#751 1 brave\n0.fire(0 6,0.5:5 0.0:5[121 0.1:5 0.2:5 0.3:5 0.4:5) 104.attr(8)\n", Default::default()).unwrap()
    }
    fn fixture_assets() -> TimedAssets {
        let mut effects = BTreeMap::new();
        let mut groups = Vec::new();
        for id in EFFECTS {
            let def = definition(id).unwrap();
            if let Some(group) = sound_group(def).unwrap() {
                groups.push(group);
            }
            let frames = (0..def.decoded_frames)
                .map(|index| SpriteFrame {
                    fragments: None,
                    x_offset: -1,
                    y_offset: -1,
                    width: 2,
                    height: 2,
                    frame_type: 0,
                    unknown1: 0,
                    unknown2: 0,
                    rgba: [index as u8 + 1, 20, 30, 255].repeat(4),
                })
                .collect();
            effects.insert(
                id,
                LoadedEffect {
                    provenance: SpriteProvenance {
                        archive: SharedArchiveKind::Sprite,
                        name: format!("{id}-0.spr"),
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
        // Empty groups do no sound I/O; these fixture pixels are not real assets.
        let sound = SoundBank::load_shared(
            "absent",
            &[],
            BankLimits::default(),
            &AtomicBool::new(false),
        )
        .unwrap();
        TimedAssets {
            effects,
            sound: Arc::new(sound),
            groups,
            catalog_sha256: [0; 32],
            encoded: 0,
            rgba: 0,
        }
    }
    #[test]
    fn exact_catalog_preserves_two_native_bracket_sounds_and_green_is_silent() {
        let cat = catalog();
        assert!(qualify(&cat, 191).unwrap().is_none());
        for (id, sound) in [(750, 142), (751, 121)] {
            let group = qualify(&cat, id).unwrap().unwrap();
            assert_eq!(group.mode, NativeAudioMode::SuppressWhileSameIdPlaying);
            assert_eq!(group.candidates(), [sound]);
        }
        for id in [0, 189, 752, 10_750, i32::MIN, i32::MAX] {
            assert!(qualify(&cat, id).is_err());
        }
    }
    #[test]
    fn one_shot_source_one_at_zero_then_terminal_zero_without_terminal_dwell() {
        let assets = fixture_assets();
        for id in EFFECTS {
            let def = definition(id).unwrap();
            let end = duration(def);
            assert_eq!(
                end,
                if id == 191 {
                    Duration::from_millis(625)
                } else {
                    Duration::from_nanos(1_041_666_667)
                }
            );
            let initial = assets.effect_frame(id, Duration::ZERO).unwrap();
            assert_eq!(initial.source_frame, 1);
            assert_eq!(initial.subframe, 0);
            assert!(!initial.finished);
            assert!(
                !assets
                    .effect_frame(id, end - Duration::from_nanos(1))
                    .unwrap()
                    .finished
            );
            let terminal = assets.effect_frame(id, end).unwrap();
            assert!(terminal.finished);
            assert_eq!(terminal.source_frame, 0);
            assert_eq!(terminal.subframe, 5);
            for (index, tick) in def.ends.iter().enumerate() {
                let elapsed = Duration::from_nanos((tick * NANOS).div_ceil(24) as u64);
                let frame = assets.effect_frame(id, elapsed).unwrap();
                assert_eq!(
                    frame.source_frame,
                    if index + 1 == def.ends.len() {
                        0
                    } else {
                        index + 2
                    }
                );
            }
            assert_eq!(assets.sprite_frames(id).unwrap().len(), def.decoded_frames);
            assert_eq!(initial.graphics_id, id as u32);
            assert_eq!(initial.sprite_name, format!("{id}-0.spr"));
        }
    }
    #[test]
    fn first_sound_crosses_once_same_time_late_sampling_and_finished_hold_never_replay() {
        let assets = fixture_assets();
        for id in EFFECTS {
            for initial_elapsed in [Duration::ZERO, Duration::MAX] {
                let mut play = assets.start_effect(id).unwrap();
                let first = play.advance(initial_elapsed).unwrap();
                let expected: Vec<CrossedCue> = definition(id)
                    .unwrap()
                    .sound
                    .map(|sound| CrossedCue {
                        source_frame: 1,
                        event_index: 0,
                        tick: 0,
                        event: FrameEvent::Sound(sound),
                    })
                    .into_iter()
                    .collect();
                assert_eq!(first.crossed_cues, expected);
                assert!(play
                    .advance(initial_elapsed)
                    .unwrap()
                    .crossed_cues
                    .is_empty());
                assert!(play.advance(Duration::MAX).unwrap().crossed_cues.is_empty());
                assert!(play.advance(Duration::ZERO).is_err());
                assert!(play.advance(Duration::MAX).unwrap().crossed_cues.is_empty());
            }
        }
    }
    #[test]
    fn different_event_has_new_sound_but_bracket_mode_is_not_changed_to_overlap() {
        let assets = fixture_assets();
        for id in [750, 751] {
            let a = assets
                .start_effect(id)
                .unwrap()
                .advance(Duration::ZERO)
                .unwrap();
            let b = assets
                .start_effect(id)
                .unwrap()
                .advance(Duration::ZERO)
                .unwrap();
            assert_eq!(a.crossed_cues, b.crossed_cues);
            assert_eq!(a.crossed_cues.len(), 1);
            assert_eq!(
                assets.sound_group(id).unwrap().unwrap().mode,
                NativeAudioMode::SuppressWhileSameIdPlaying
            );
        }
        assert!(assets.sound_group(191).unwrap().is_none());
        assert_eq!(assets.sound_groups().len(), 2);
    }
    #[test]
    fn no_hit_caret_lt_audio_new_sound_or_relocated_sound_can_silently_qualify() {
        for event in [
            FrameEvent::Sound(119),
            FrameEvent::Sound(10_142),
            FrameEvent::Effect(142),
            FrameEvent::Hit,
            FrameEvent::Overlay(750),
            FrameEvent::Skip,
            FrameEvent::UninterpretedNumeric {
                marker: b'^',
                value: 142,
            },
        ] {
            let mut cat = catalog();
            let ActionDefinition::Animation(animation) = cat
                .sprites
                .get_mut(&750)
                .unwrap()
                .actions
                .get_mut(&ActionKey::new(0))
                .unwrap()
            else {
                panic!("fixture");
            };
            animation.frames[1].events = vec![event];
            assert!(qualify(&cat, 750).is_err());
        }
        for frame in [0, 2, 3, 4, 5] {
            let mut cat = catalog();
            let ActionDefinition::Animation(animation) = cat
                .sprites
                .get_mut(&751)
                .unwrap()
                .actions
                .get_mut(&ActionKey::new(0))
                .unwrap()
            else {
                panic!("fixture");
            };
            animation.frames[1].events.clear();
            animation.frames[frame].events.push(FrameEvent::Sound(121));
            assert!(qualify(&cat, 751).is_err());
        }
        let mut missing = catalog();
        let ActionDefinition::Animation(animation) = missing
            .sprites
            .get_mut(&750)
            .unwrap()
            .actions
            .get_mut(&ActionKey::new(0))
            .unwrap()
        else {
            panic!("fixture");
        };
        animation.frames[1].events.clear();
        assert!(qualify(&missing, 750).is_err());
        let mut cat = catalog();
        let ActionDefinition::Animation(animation) = cat
            .sprites
            .get_mut(&191)
            .unwrap()
            .actions
            .get_mut(&ActionKey::new(0))
            .unwrap()
        else {
            panic!("fixture");
        };
        animation.frames[1].events.push(FrameEvent::Sound(121));
        assert!(qualify(&cat, 191).is_err());
    }
    #[test]
    fn changed_fps_direction_frame_bank_attributes_and_aliases_fail_closed() {
        for mutation in 0..8 {
            let mut cat = catalog();
            let sprite = cat.sprites.get_mut(&750).unwrap();
            match mutation {
                0 => sprite.graphics_id = 751,
                1 => sprite.image_count = Some(2),
                2 => sprite.attributes.clear(),
                3 => sprite.attributes[0].body_bytes = b"9".to_vec(),
                _ => {
                    let ActionDefinition::Animation(animation) =
                        sprite.actions.get_mut(&ActionKey::new(0)).unwrap()
                    else {
                        panic!("fixture");
                    };
                    match mutation {
                        4 => animation.frames_per_second = 25,
                        5 => animation.directionality = Directionality::EightWay,
                        6 => animation.frames[1].duration_ticks = 4,
                        _ => animation.frames[5].subframe = 10,
                    }
                }
            }
            assert!(qualify(&cat, 750).is_err());
        }
        let mut cat = catalog();
        cat.sprites.get_mut(&750).unwrap().actions.insert(
            ActionKey::new(0),
            ActionDefinition::Reference {
                sprite_id: 751,
                label_bytes: Vec::new(),
            },
        );
        assert!(qualify(&cat, 750).is_err());
    }
    #[test]
    fn draw_is_memory_only_at_event_origin_and_terminal_does_not_change_pixels() {
        let assets = fixture_assets();
        for id in EFFECTS {
            let mut canvas = Canvas::new(8, 8).unwrap();
            let baseline = canvas.rgba().to_vec();
            assert!(assets
                .draw_effect(&mut canvas, id, PixelPoint { x: 4, y: 4 }, Duration::ZERO)
                .unwrap());
            assert_ne!(canvas.rgba(), baseline);
            let mut ended = Canvas::new(8, 8).unwrap();
            assert!(!assets
                .draw_effect(
                    &mut ended,
                    id,
                    PixelPoint { x: 4, y: 4 },
                    assets.duration(id).unwrap()
                )
                .unwrap());
            assert_eq!(ended.rgba(), baseline);
        }
    }
    #[test]
    fn cancelled_load_checks_before_any_path_or_asset_read() {
        assert!(matches!(
            TimedAssets::load(Path::new("not-a-shared-root"), &AtomicBool::new(true)),
            Err(TimedError::Cancelled)
        ));
    }
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; actual191/750/751 screen layers, memory-only independent floor reference"]
    fn actual_timed_attr8_layers_use_screen_and_retire_without_replaying_cues() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        let assets = TimedAssets::load(Path::new(&root), &AtomicBool::new(false)).unwrap();
        let anchor = PixelPoint { x: 400, y: 350 };
        for id in EFFECTS {
            let duration = assets.duration(id).unwrap();
            let mut changed = (false, false);
            for elapsed in [
                Duration::ZERO,
                Duration::from_millis(300),
                duration - Duration::from_nanos(1),
            ] {
                let sample = assets.effect_frame(id, elapsed).unwrap();
                let result = super::super::teleport_effect::screen_test_support::compare(
                    sample.frame,
                    anchor,
                    |canvas| assets.draw_effect(canvas, id, anchor, elapsed).unwrap(),
                );
                changed.0 |= result.0;
                changed.1 |= result.1;
            }
            assert_eq!(
                changed,
                (true, true),
                "actual timed{id} must differ from ordinary opaque drawing"
            );
            super::super::teleport_effect::screen_test_support::retired(|canvas| {
                assets.draw_effect(canvas, id, anchor, duration).unwrap()
            });
            let mut playback = assets.start_effect(id).unwrap();
            let first = playback.advance(Duration::ZERO).unwrap();
            assert_eq!(first.crossed_cues.len(), usize::from(id != 191));
            assert!(playback
                .advance(Duration::ZERO)
                .unwrap()
                .crossed_cues
                .is_empty());
            assert!(playback.advance(duration).unwrap().crossed_cues.is_empty());
            assert!(playback
                .advance(Duration::MAX)
                .unwrap()
                .crossed_cues
                .is_empty());
        }
    }
    #[test]
    fn unknown_effects_and_missing_decoded_frames_never_use_a_substitute() {
        let mut assets = fixture_assets();
        for id in [0, 189, 194, 197, 10_750, -1, i32::MAX] {
            assert!(assets.start_effect(id).is_err());
            assert!(assets.duration(id).is_err());
            assert!(assets.effect_frame(id, Duration::ZERO).is_err());
            assert!(assets.sound_group(id).is_err());
            assert!(assets.sprite_frames(id).is_err());
        }
        assets.effects.get_mut(&750).unwrap().decoded.frames.clear();
        assert!(assets.effect_frame(750, Duration::ZERO).is_err());
        assert!(assets.effect_frame(750, Duration::MAX).is_err());
    }
}
