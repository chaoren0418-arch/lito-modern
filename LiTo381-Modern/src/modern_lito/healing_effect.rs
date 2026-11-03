//! Shared-only potion presentation for qualified effects 189, 194 and 197.
//!
//! Shared Text/list.spr SHA256 523c61ad...b9f127 gives each effect exactly
//! action0: `0.5:3 0.0:1 0.1:1 0.2:1 0.3:2 0.4:2 0.3:2 0.4:2 0.3:2 0.4:2`,
//! undirected, default24FPS, attr104=8. There are NO sound or Hit markers.
//! Native effect one-shot sub_44EF80 -> sub_4E5B50 plays source indices1..9
//! followed by terminal0, without dwelling there: 15ticks, exactly625ms.
//! These are anchored overlays, not actor replacements, projectiles, healing
//! calculations or inventory commands. The caller supplies the confirmed use
//! event's projected position and owns the bounded simultaneous-effect list.
//!
//! Loading is worker-only. All three SPRs and all referenced subframes must
//! qualify before publication. Reads/decode are bounded; cancellation is
//! cooperative between operations, not an interrupt of a filesystem call.
//! Immutable loaded assets retain no file handles or workers. Rendering is
//! memory-only. Native hash-shard routing permits base Sprite fallback ONLY
//! when the selected shard lacks the member, never on corrupt/ambiguous data.
//! No loose, LT, guessed WAV, unknown opcode or substitute-graphics fallback.

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

use sha2::{Digest, Sha256};

use super::{
    combat_animation::{CrossedCue, NativeAudioGroup},
    combat_audio::{BankLimits, SoundBank},
    compositor::Canvas,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    shared_action_catalog::{SharedActionCatalog, SharedCatalogError},
    sprite::{decode_classic_spr_with_limits, DecodedSprite, SpriteError, SpriteLimits},
    sprite_actions::{ActionError, ActionKey, Directionality, SpriteActionCatalog},
    teleport_effect::{draw_screen_frame, TeleportError},
    world_map::PixelPoint,
};

pub use super::teleport_effect::{SpriteProvenance, VisualSample};

pub const EFFECTS: [i32; 3] = [189, 194, 197];
const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 24 * 1024 * 1024;
const MAX_RGBA: usize = 32 * 1024 * 1024;
const MAX_INDEX_RECORDS: usize = 1_000_000;
const SUBFRAMES: [usize; 10] = [5, 0, 1, 2, 3, 4, 3, 4, 3, 4];
const DURATIONS: [u32; 10] = [3, 1, 1, 1, 2, 2, 2, 2, 2, 2];
const ACTIVE_ENDS: [u128; 9] = [1, 2, 3, 5, 7, 9, 11, 13, 15];
const NANOS: u128 = 1_000_000_000;
const DURATION: Duration = Duration::from_millis(625);

#[derive(Debug)]
pub enum HealingError {
    Cancelled,
    InvalidInput(&'static str),
    Unsupported(&'static str),
    Bound(&'static str),
    Resource(ResourceError),
    Catalog(ActionError),
    Sprite(SpriteError),
    Render(TeleportError),
}
impl fmt::Display for HealingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "healing resource loading cancelled"),
            Self::InvalidInput(reason) => write!(f, "invalid healing presentation: {reason}"),
            Self::Unsupported(reason) => {
                write!(f, "unsupported shared healing presentation: {reason}")
            }
            Self::Bound(reason) => write!(f, "healing presentation exceeds {reason} bound"),
            Self::Resource(error) => write!(f, "healing shared resource: {error}"),
            Self::Catalog(error) => write!(f, "healing action definition: {error}"),
            Self::Sprite(error) => write!(f, "healing SPR: {error}"),
            Self::Render(error) => write!(f, "healing drawing: {error}"),
        }
    }
}
impl std::error::Error for HealingError {}
impl From<SharedCatalogError> for HealingError {
    fn from(error: SharedCatalogError) -> Self {
        match error {
            SharedCatalogError::Cancelled => Self::Cancelled,
            SharedCatalogError::Bound(reason) => Self::Bound(reason),
            SharedCatalogError::Resource(error) => Self::Resource(error),
            SharedCatalogError::Action(error) => Self::Catalog(error),
        }
    }
}
impl From<ResourceError> for HealingError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
impl From<ActionError> for HealingError {
    fn from(value: ActionError) -> Self {
        Self::Catalog(value)
    }
}
impl From<SpriteError> for HealingError {
    fn from(value: SpriteError) -> Self {
        Self::Sprite(value)
    }
}
type Result<T> = std::result::Result<T, HealingError>;

fn active(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(HealingError::Cancelled)
    } else {
        Ok(())
    }
}
fn known(effect: i32) -> Result<u32> {
    if EFFECTS.contains(&effect) {
        Ok(effect as u32)
    } else {
        Err(HealingError::Unsupported(
            "effect ID is not one of 189/194/197",
        ))
    }
}

/// No animation event may be silently dropped. This deliberately qualifies a
/// small exact, cue-free definition instead of generalizing unknown controls.
fn qualify(catalog: &SpriteActionCatalog, effect: i32) -> Result<()> {
    let id = known(effect)?;
    let sprite = catalog
        .sprites
        .get(&id)
        .ok_or(HealingError::Unsupported("missing healing definition"))?;
    let resolved = catalog.resolve_action(id, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != id
        || sprite.image_count != Some(1)
        || resolved.definition_sprite_id != id
        || sprite.actions.len() != 1
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != SUBFRAMES.len()
        || sprite.attributes.len() != 1
        || sprite.attributes[0].key != ActionKey::new(104)
        || sprite.attributes[0].body_bytes != b"8"
    {
        return Err(HealingError::Unsupported(
            "unverified healing controls or graphics",
        ));
    }
    for (index, frame) in animation.frames.iter().enumerate() {
        if frame.image_id != 0
            || frame.subframe as usize != SUBFRAMES[index]
            || frame.duration_ticks != DURATIONS[index]
            || !frame.events.is_empty()
        {
            return Err(HealingError::Unsupported(
                "unverified healing frames or cues",
            ));
        }
        let asset = animation.frame_asset(id, index, 0)?;
        if asset.graphics_id != id || asset.image_id != 0 {
            return Err(HealingError::Unsupported("unverified healing SPR route"));
        }
    }
    Ok(())
}

fn sample(elapsed: Duration) -> Result<(usize, bool)> {
    let tick = elapsed
        .as_nanos()
        .checked_mul(24)
        .ok_or(HealingError::Bound("elapsed time"))?
        / NANOS;
    if tick >= 15 {
        Ok((0, true))
    } else {
        Ok((ACTIVE_ENDS.partition_point(|end| *end <= tick) + 1, false))
    }
}

pub struct HealingSample {
    pub source_frame: usize,
    pub finished: bool,
    /// Empty for all three qualified definitions. Unknown cues fail loading;
    /// they are never converted to healing commands or guessed audio.
    pub crossed_cues: Vec<CrossedCue>,
}
pub struct HealingPlayback {
    last_elapsed: Option<Duration>,
}
impl HealingPlayback {
    pub fn advance(&mut self, elapsed: Duration) -> Result<HealingSample> {
        if self.last_elapsed.is_some_and(|last| elapsed < last) {
            return Err(HealingError::InvalidInput("elapsed time regressed"));
        }
        let (source_frame, finished) = sample(elapsed)?;
        self.last_elapsed = Some(elapsed);
        Ok(HealingSample {
            source_frame,
            finished,
            crossed_cues: Vec::new(),
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
    fn open(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        active(cancel)?;
        if base.len() > MAX_INDEX_RECORDS {
            return Err(HealingError::Bound("index records"));
        }
        Ok(Self {
            root: base.root().to_owned(),
            records: base.len(),
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
                .ok_or(HealingError::Bound("index records"))?;
            if self.records > MAX_INDEX_RECORDS {
                return Err(HealingError::Bound("index records"));
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

pub struct HealingAssets {
    effects: BTreeMap<i32, LoadedEffect>,
    sound: Arc<SoundBank>,
    catalog_sha256: [u8; 32],
    encoded: usize,
    rgba: usize,
}
impl HealingAssets {
    /// Worker-only all-or-nothing loading of exactly three shared effects.
    /// <=8MiB encoded per entry, <=24MiB total encoded, <=32MiB decoded RGBA.
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
        let size = context.byte_length();
        if context.index_records() > MAX_INDEX_RECORDS || size == 0 || size > MAX_ENTRY {
            return Err(HealingError::Bound("action catalog"));
        }
        active(cancel)?;
        let catalog_sha256 = context.sha256();
        let catalog = context.catalog();
        for effect in EFFECTS {
            active(cancel)?;
            qualify(&catalog, effect)?;
        }
        let mut reader = SpriteReader::open(root, cancel)?;
        let mut effects = BTreeMap::new();
        let mut encoded = 0usize;
        let mut rgba = 0usize;
        for effect in EFFECTS {
            active(cancel)?;
            let name = format!("{effect}-0.spr");
            let archive = reader.archive_for(&name, cancel)?;
            let size = archive.entry(&name)?.size as usize;
            if size == 0 || size > MAX_ENTRY || size > MAX_ENCODED - encoded {
                return Err(HealingError::Bound("encoded sprites"));
            }
            let kind = archive.kind();
            active(cancel)?;
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            if bytes.len() != size {
                return Err(HealingError::Unsupported("SPR length changed"));
            }
            let remaining = (MAX_RGBA - rgba) / 4;
            if remaining == 0 {
                return Err(HealingError::Bound("decoded sprites"));
            }
            let decoded = decode_classic_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_frames: 6,
                    max_total_pixels: remaining,
                    ..Default::default()
                },
            )?;
            active(cancel)?;
            if decoded.frames.len() != 6 {
                return Err(HealingError::Unsupported(
                    "missing or extra healing SPR subframes",
                ));
            }
            let count = decoded
                .frames
                .iter()
                .try_fold(0usize, |sum, frame| sum.checked_add(frame.rgba.len()))
                .ok_or(HealingError::Bound("decoded sprites"))?;
            if count > MAX_RGBA - rgba {
                return Err(HealingError::Bound("decoded sprites"));
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
        // Empty groups perform no sound I/O. A nonempty cue definition never
        // reaches this point: qualify() rejects it instead of silently muting.
        let sound = SoundBank::load_shared(
            root,
            &[],
            BankLimits {
                max_bytes: 1,
                max_sounds: 1,
                max_groups: 1,
            },
            cancel,
        )
        .map_err(|_| {
            if cancel.load(Ordering::Acquire) {
                HealingError::Cancelled
            } else {
                HealingError::Unsupported("empty sound bank")
            }
        })?;
        active(cancel)?;
        Ok(Self {
            effects,
            sound: Arc::new(sound),
            catalog_sha256,
            encoded,
            rgba,
        })
    }
    fn effect(&self, effect: i32) -> Result<&LoadedEffect> {
        known(effect)?;
        self.effects
            .get(&effect)
            .ok_or(HealingError::Unsupported("effect was not preloaded"))
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.catalog_sha256
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.effects.values().map(|effect| &effect.provenance)
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
    pub fn sound_group(&self, effect: i32) -> Result<Option<&NativeAudioGroup>> {
        self.effect(effect)?;
        Ok(None)
    }
    pub fn sound_groups(&self) -> impl ExactSizeIterator<Item = &NativeAudioGroup> {
        std::iter::empty()
    }
    pub fn duration(&self, effect: i32) -> Result<Duration> {
        self.effect(effect)?;
        Ok(DURATION)
    }
    pub fn start_effect(&self, effect: i32) -> Result<HealingPlayback> {
        self.effect(effect)?;
        Ok(HealingPlayback { last_elapsed: None })
    }
    pub fn effect_frame(&self, effect: i32, elapsed: Duration) -> Result<VisualSample<'_>> {
        let loaded = self.effect(effect)?;
        let (source_frame, finished) = sample(elapsed)?;
        let subframe = SUBFRAMES[source_frame];
        let frame = loaded
            .decoded
            .frames
            .get(subframe)
            .ok_or(HealingError::Unsupported("unloaded healing subframe"))?;
        Ok(VisualSample {
            frame,
            source_frame,
            finished,
            graphics_id: effect as u32,
            sprite_name: &loaded.provenance.name,
            subframe,
        })
    }
    /// Caller keeps this exact authoritative use event's projected origin;
    /// subsequent actor movement does not move an already emitted potion cue.
    /// A terminal frame remains inspectable but is not left on the canvas.
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
        draw_screen_frame(canvas, sample.frame, anchor).map_err(HealingError::Render)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::super::sprite_actions::{ActionDefinition, FrameEvent};
    use super::*;

    fn catalog() -> SpriteActionCatalog {
        let mut text = String::from("3 0 0\n");
        for id in EFFECTS {
            text.push_str(&format!("#{id} 1 potion\n0.fire(0 10,0.5:3 0.0:1 0.1:1 0.2:1 0.3:2 0.4:2 0.3:2 0.4:2 0.3:2 0.4:2) 104.attr(8)\n"));
        }
        SpriteActionCatalog::parse_for_inspection(text.as_bytes(), Default::default()).unwrap()
    }
    fn fixture_assets() -> HealingAssets {
        let mut effects = BTreeMap::new();
        for id in EFFECTS {
            let frames = (0..6)
                .map(|index| super::super::sprite::SpriteFrame {
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
        HealingAssets {
            effects,
            sound: Arc::new(
                SoundBank::load_shared(
                    "absent",
                    &[],
                    BankLimits::default(),
                    &AtomicBool::new(false),
                )
                .unwrap(),
            ),
            catalog_sha256: [0; 32],
            encoded: 0,
            rgba: 0,
        }
    }
    #[test]
    fn exact_shared_definitions_are_qualified_without_cues() {
        let cat = catalog();
        for id in EFFECTS {
            qualify(&cat, id).unwrap();
        }
        for id in [0, 169, 188, 195, 10_189, i32::MIN, i32::MAX] {
            assert!(qualify(&cat, id).is_err());
        }
    }
    #[test]
    fn one_shot_plays_active_indices_and_removes_terminal_without_dwell() {
        let assets = fixture_assets();
        for id in EFFECTS {
            assert_eq!(assets.duration(id).unwrap(), Duration::from_millis(625));
            let mut play = assets.start_effect(id).unwrap();
            for (tick, expected) in [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 4),
                (5, 5),
                (7, 6),
                (9, 7),
                (11, 8),
                (13, 9),
                (14, 9),
                (15, 0),
            ] {
                let elapsed = Duration::from_nanos((tick * 1_000_000_000u64).div_ceil(24));
                let current = play.advance(elapsed).unwrap();
                assert_eq!(current.source_frame, expected);
                assert_eq!(current.finished, tick == 15);
                assert!(current.crossed_cues.is_empty());
                let frame = assets.effect_frame(id, elapsed).unwrap();
                assert_eq!(frame.subframe, SUBFRAMES[expected]);
                assert_eq!(frame.graphics_id, id as u32);
                assert_eq!(frame.sprite_name, format!("{id}-0.spr"));
            }
            assert!(
                !assets
                    .effect_frame(id, Duration::from_nanos(624_999_999))
                    .unwrap()
                    .finished
            );
            assert!(play.advance(Duration::MAX).unwrap().finished);
            assert!(play.advance(Duration::MAX).unwrap().crossed_cues.is_empty());
            assert!(play.advance(Duration::ZERO).is_err());
        }
    }
    #[test]
    fn unverified_audio_hit_frames_attributes_and_routes_fail_closed() {
        for event in [
            FrameEvent::Sound(119),
            FrameEvent::Effect(478),
            FrameEvent::Hit,
            FrameEvent::Overlay(189),
            FrameEvent::UninterpretedNumeric {
                marker: b'^',
                value: 1,
            },
            FrameEvent::Skip,
        ] {
            let mut cat = catalog();
            let ActionDefinition::Animation(animation) = cat
                .sprites
                .get_mut(&189)
                .unwrap()
                .actions
                .get_mut(&ActionKey::new(0))
                .unwrap()
            else {
                panic!("fixture animation");
            };
            animation.frames[1].events.push(event);
            assert!(qualify(&cat, 189).is_err());
        }
        for mutation in 0..8 {
            let mut cat = catalog();
            let sprite = cat.sprites.get_mut(&189).unwrap();
            match mutation {
                0 => sprite.graphics_id = 194,
                1 => sprite.image_count = Some(2),
                2 => sprite.attributes[0].body_bytes = b"9".to_vec(),
                3 => sprite.attributes.clear(),
                _ => {
                    let ActionDefinition::Animation(animation) =
                        sprite.actions.get_mut(&ActionKey::new(0)).unwrap()
                    else {
                        panic!("fixture animation");
                    };
                    match mutation {
                        4 => animation.frames_per_second = 25,
                        5 => animation.directionality = Directionality::EightWay,
                        6 => animation.frames[0].duration_ticks = 4,
                        _ => animation.frames[9].subframe = 5,
                    }
                }
            }
            assert!(qualify(&cat, 189).is_err());
        }
        let mut aliased = catalog();
        aliased.sprites.get_mut(&189).unwrap().actions.insert(
            ActionKey::new(0),
            ActionDefinition::Reference {
                sprite_id: 194,
                label_bytes: Vec::new(),
            },
        );
        assert!(qualify(&aliased, 189).is_err());
    }
    #[test]
    fn no_guessed_sound_no_io_on_precancel_and_unknown_effect_rejected() {
        assert!(matches!(
            HealingAssets::load(Path::new("absent"), &AtomicBool::new(true)),
            Err(HealingError::Cancelled)
        ));
        let assets = fixture_assets();
        assert!(assets.sound_bank().is_empty());
        assert_eq!(assets.sound_bank().file_bytes(), 0);
        assert_eq!(assets.sound_groups().len(), 0);
        for id in EFFECTS {
            assert!(assets.sound_group(id).unwrap().is_none());
        }
        assert!(assets.sound_group(169).is_err());
        assert!(assets.start_effect(169).is_err());
        assert!(assets.duration(169).is_err());
        assert!(assets.effect_frame(169, Duration::ZERO).is_err());
    }
    #[test]
    fn fixed_origin_clipping_and_completed_draw_are_memory_only() {
        let assets = fixture_assets();
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.clear([0, 0, 0]);
        assert!(assets
            .draw_effect(&mut canvas, 189, PixelPoint { x: 0, y: 0 }, Duration::ZERO)
            .unwrap());
        assert_eq!(&canvas.rgba()[..4], &[0, 16, 24, 255]); // explicit RGB555 source quantization
        assert_eq!(&canvas.rgba()[4..8], &[0, 0, 0, 255]);
        let before = canvas.rgba().to_vec();
        assert!(!assets
            .draw_effect(&mut canvas, 189, PixelPoint { x: 2, y: 2 }, DURATION)
            .unwrap());
        assert_eq!(canvas.rgba(), before);
        assert!(assets
            .draw_effect(
                &mut canvas,
                194,
                PixelPoint { x: i32::MIN, y: 0 },
                Duration::ZERO
            )
            .is_err());
    }
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; reads shared189/194/197 and compares independent C# RGBA, no files/audio playback"]
    fn actual_shared_healing_effects_match_all_eighteen_reference_frames() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        let assets = HealingAssets::load(Path::new(&root), &AtomicBool::new(false)).unwrap();
        assert_eq!(
            assets.catalog_sha256(),
            hex("523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127")
        );
        assert_eq!(assets.sprite_sources().len(), 3);
        assert_eq!(assets.encoded_bytes(), 186_672);
        assert_eq!(assets.rgba_bytes(), 1_709_568);
        assert!(assets.sound_bank().is_empty());
        assert_eq!(assets.sound_bank().file_bytes(), 0);
        assert_eq!(assets.sound_groups().len(), 0);
        let raw = [
            (
                69_956,
                "7DD63E84350E07D451D3D07D4F0F9463BFEB7C367E3BC07AE90D6DC297B35845",
            ),
            (
                58_359,
                "EC825A14A3E982AEC43AF29A3B1A6CAD7BB8FE603E8DCE81080AB87BF50887ED",
            ),
            (
                58_357,
                "F22769B733A2B0CA94128DA3EA824753C262DB690DA22A1F84DA5A714A92DD56",
            ),
        ];
        // Independent existing PakViewer SprReader.LoadRaw, in-memory only.
        // Actual base entries end in uppercase .SPR; canonical lookup is
        // case-insensitive. Their selected shards genuinely lack the member.
        let hashes = [
            [
                "9468AFA1D5C6092D382F30CDAD2891B89DA5FAB10A672AB7F6AB8FC43C1AC29A",
                "24632288DF89A1884E3F8603594D406F160279FCB5FCCC71DEC20BBA30DEE6A7",
                "A3D6BC12D3F57EC15426D9FAFDA801AE2796EB2030581B48A79A5735AF92E251",
                "C4ACB30AB5E988A4B66019C9A8347E0A27CD9B177A7E83471AD8D47123F9F474",
                "E83C5109576B44CE0E89EAA776E0C4E830531890AAF9A082B4B08BD644E90249",
                "8E3F52B718BA38C255C563C468313B17904838D297AF98C550853BFA16C899C5",
            ],
            [
                "F9056AEC6556323618FA68E35BC6274FFD0F3C08B63C72AA47828A13CF0D5078",
                "13B4C85961EB5B8B93F2111C8B38C913FE5337894CAACCB96156E4748F567BC7",
                "A3A58FC1694AD4DDCB1BC20D5AA3FD4A3DE10E71E605F0A6E80BE957F1E61A3F",
                "12B44235027CFF4E5853F3D07AD58C250B12F8E93BB668BF56F441D23E29FBD6",
                "DD908ECA25C10EA1A12E7725B9BEE87044CE0FE7708916651FCFB39238DC80AB",
                "44446BB4FA068E6D6B7AF87814EEABE2F8C275C3BA70FB0653BABBD0113AEB29",
            ],
            [
                "D2A17B67A3D7C3B7D1B3881D7E780299B72C3E20E27308F10C946AA782D2689F",
                "62BF559796C9B1F7AC39E20A02BD64562EE0257C1D347B5BBA25F7685EB67AEA",
                "587A69CA7DE14A3CF78AE4BB3AADB79A73F2A627487684E66A2E481DFF83EEBF",
                "3C606ECCB7059750069A81FC2427D5F12183AD088E0CCD7ADF7D1A89416004E7",
                "692C20EA6BAA362461C30F91538271D5BC12C5C400D93C6569157DB8BCC8B813",
                "788A9B970B1E1DF2D5901469311FE58656841716C33B6BB89E64B54036397928",
            ],
        ];
        let red_bounds = [
            (-72, -120, 192, 180),
            (-72, -108, 192, 168),
            (-72, -120, 192, 180),
            (-72, -96, 168, 144),
            (-48, -84, 144, 120),
            (-24, -72, 96, 96),
        ];
        let other_bounds = [
            (-48, -108, 144, 156),
            (-72, -108, 192, 168),
            (-72, -120, 192, 168),
            (-72, -96, 168, 144),
            (-48, -84, 144, 120),
            (-24, -72, 96, 96),
        ];
        for (index, id) in EFFECTS.into_iter().enumerate() {
            let effect = &assets.effects[&id];
            assert_eq!(effect.provenance.archive, SharedArchiveKind::Sprite);
            assert_eq!(effect.provenance.name, format!("{id}-0.spr"));
            assert_eq!(effect.provenance.bytes, raw[index].0);
            assert_eq!(effect.provenance.sha256, hex(raw[index].1));
            assert_eq!(effect.decoded.frames.len(), 6);
            let bounds = if index == 0 {
                &red_bounds
            } else {
                &other_bounds
            };
            for (subframe, frame) in effect.decoded.frames.iter().enumerate() {
                assert_eq!(
                    (frame.x_offset, frame.y_offset, frame.width, frame.height),
                    bounds[subframe]
                );
                let pixel_hash: [u8; 32] = Sha256::digest(&frame.rgba).into();
                assert_eq!(pixel_hash, hex(hashes[index][subframe]));
            }
            assert_eq!(assets.duration(id).unwrap(), DURATION);
            assert!(assets.sound_group(id).unwrap().is_none());
            let mut playback = assets.start_effect(id).unwrap();
            for elapsed in [
                Duration::ZERO,
                Duration::from_millis(500),
                DURATION,
                Duration::MAX,
            ] {
                assert!(playback.advance(elapsed).unwrap().crossed_cues.is_empty());
            }
            let mut canvas = Canvas::new(800, 600).unwrap();
            canvas.clear([0, 0, 0]);
            assert!(assets
                .draw_effect(
                    &mut canvas,
                    id,
                    PixelPoint { x: 400, y: 300 },
                    Duration::ZERO
                )
                .unwrap());
            let displayed = canvas.rgba().to_vec();
            assert!(!assets
                .draw_effect(&mut canvas, id, PixelPoint { x: 400, y: 300 }, DURATION)
                .unwrap());
            assert_eq!(canvas.rgba(), displayed);
            let anchor = PixelPoint { x: 400, y: 350 };
            let mut changed = (false, false);
            for ms in [0, 100, 300, 624] {
                let elapsed = Duration::from_millis(ms);
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
                "actual healing{id} screen must differ from opaque pixels"
            );
            super::super::teleport_effect::screen_test_support::retired(|canvas| {
                assets.draw_effect(canvas, id, anchor, DURATION).unwrap()
            });
        }
    }
    fn hex(value: &str) -> [u8; 32] {
        std::array::from_fn(|i| u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).unwrap())
    }
}
