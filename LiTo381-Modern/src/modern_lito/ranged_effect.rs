//! Shared ordinary-arrow66, native projectile class3. No actor, damage or audio.
//!
//! Evidence: ida-client-oep-v2/full-disassembly.asm sub_40E8F0 (29604)
//! snapshots the live target ON LAUNCH for a hit; a miss keeps packet coords.
//! sub_4B92D0 (386862) builds the 16.16 vector, sub_40E420 (29024) performs
//! the first step immediately, and sub_40E8D0 (29584) repeats at30ms without
//! retargeting. Caller freezes the endpoint at the bow21 first Hit marker:
//! latest SERVER-CONFIRMED target/corpse for a hit, event endpoint for a miss.
//!
//! Class3 keeps all8 headings. Its remaining counter is max(2*abs(dx),abs(dy)),
//! not the vector's max(abs(dx),abs(dy)) denominator. At29446 it sets frame
//! index0/1/2 according to remaining>1/==1/<=0, then4E5B50 increments before
//! loading. Thus source1/sub0, source2/sub1, terminal0/sub2. The terminal frame
//! is immediately unregistered/deleted (29513..29562), never held or drawn.
//! Adjacent and zero-distance arrows therefore finish in the first callback.
//! No guessed impact SPR, Sound or24FPS loop is added. This path has no actor
//! side effects; native_projectiles releases presentation hurt on arrival.
//!
//! The caller owns per-shot lifetimes and discards them on world-generation or
//! entity departure. Assets load only in a worker; immutable draw/sample has no
//! I/O, handles, threads, audio cursor, target lookup or combat side effects.

use std::{
    collections::BTreeMap,
    fmt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use sha2::{Digest, Sha256};

use super::{
    compositor::Canvas,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    shared_action_catalog::{SharedActionCatalog, SharedCatalogError},
    sprite::{decode_world_spr_with_limits, DecodedSprite, SpriteError, SpriteFrame, SpriteLimits},
    sprite_actions::{ActionError, ActionKey, Animation, Directionality, SpriteActionCatalog},
    world_map::{ViewProjection, WorldCell},
};

const GRAPHICS_ID: u32 = 66;
const STEP_NANOS: u64 = 30_000_000;
/// Presentation limit, NOT the authority's attack range. A target can move
/// between shot acceptance and visual launch. Omit an over-bound presentation;
/// do not fail the world or change the already committed hit/ammunition.
const MAX_FLIGHT_CELLS: i64 = 256;
const MAX_CATALOG: usize = 8 * 1024 * 1024;
const MAX_ENTRY: usize = 1024 * 1024;
const MAX_ENCODED: usize = 8 * MAX_ENTRY;
const MAX_RGBA: usize = 8 * 1024 * 1024;
const MAX_INDEX_RECORDS: usize = 1_000_000;

#[derive(Debug)]
pub enum RangedError {
    Cancelled,
    InvalidInput(&'static str),
    Unsupported(&'static str),
    Bound(&'static str),
    Resource(ResourceError),
    Catalog(ActionError),
    Sprite(SpriteError),
    Render(anyhow::Error),
}
impl fmt::Display for RangedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "ordinary arrow loading cancelled"),
            Self::InvalidInput(reason) => {
                write!(f, "invalid ordinary arrow presentation: {reason}")
            }
            Self::Unsupported(reason) => write!(f, "unsupported shared ordinary arrow: {reason}"),
            Self::Bound(reason) => write!(f, "ordinary arrow exceeds {reason} bound"),
            Self::Resource(error) => write!(f, "ordinary arrow shared resource: {error}"),
            Self::Catalog(error) => write!(f, "ordinary arrow action definition: {error}"),
            Self::Sprite(error) => write!(f, "ordinary arrow SPR: {error}"),
            Self::Render(error) => write!(f, "ordinary arrow drawing: {error}"),
        }
    }
}
impl std::error::Error for RangedError {}
impl From<SharedCatalogError> for RangedError {
    fn from(error: SharedCatalogError) -> Self {
        match error {
            SharedCatalogError::Cancelled => Self::Cancelled,
            SharedCatalogError::Bound(reason) => Self::Bound(reason),
            SharedCatalogError::Resource(error) => Self::Resource(error),
            SharedCatalogError::Action(error) => Self::Catalog(error),
        }
    }
}
impl From<ResourceError> for RangedError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
impl From<ActionError> for RangedError {
    fn from(value: ActionError) -> Self {
        Self::Catalog(value)
    }
}
impl From<SpriteError> for RangedError {
    fn from(value: SpriteError) -> Self {
        Self::Sprite(value)
    }
}
type Result<T> = std::result::Result<T, RangedError>;

fn active(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(RangedError::Cancelled)
    } else {
        Ok(())
    }
}
fn reserve<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| RangedError::Bound("allocation"))?;
    Ok(values)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangedSample {
    pub position: WorldCell,
    pub heading: u8,
    /// Index in list.spr's3-frame definition, not the SPR subframe index.
    pub source_frame: usize,
    pub finished: bool,
}

/// One immutable launch-time path. Its precomputed samples are bounded by512
/// callbacks. Sampling a large elapsed time is O(1), without catch-up work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RangedPath {
    from: WorldCell,
    target: WorldCell,
    heading: u8,
    steps: Vec<RangedSample>,
}
impl RangedPath {
    pub fn new(from: WorldCell, target: WorldCell) -> Result<Self> {
        for cell in [from, target] {
            // Original positions become unsigned16 halves of a32-bit16.16
            // accumulator. Reject an unrepresented nativeX rather than wrap
            // to another world cell. All qualified current map514 cells fit.
            if !(16384..=49151).contains(&cell.x) || !(0..=65535).contains(&cell.y) {
                return Err(RangedError::Bound("native fixed-point coordinates"));
            }
        }
        let dx = i64::from(target.x) - i64::from(from.x);
        let dy = i64::from(target.y) - i64::from(from.y);
        let distance = dx.abs().max(dy.abs());
        if distance > MAX_FLIGHT_CELLS {
            return Err(RangedError::Bound("projectile presentation lifetime"));
        }
        let heading = if distance == 0 {
            0
        } else {
            native_heading(dx, dy)
        };
        let mut remaining = (2 * dx.abs()).max(dy.abs());
        let mut steps = reserve(remaining.max(1) as usize)?;
        if distance == 0 {
            // Zero vector, counter0, first callback reaches the endpoint and
            // deletes the ordinary projectile. No flight or impact bitmap.
            steps.push(RangedSample {
                position: target,
                heading,
                source_frame: 0,
                finished: true,
            });
        } else {
            // Native divides BEFORE doubling x velocity; signed division
            // truncates toward zero. Use wider exact arithmetic, never lerp.
            let vx = (dx * 65536 / distance) * 2;
            let vy = dy * 65536 / distance;
            let mut x = (2 * i64::from(from.x) - 32768) * 65536;
            let mut y = i64::from(from.y) * 65536;
            while remaining > 0 {
                x += vx;
                y += vy;
                let native_x = (x + 32768).div_euclid(65536) & !1;
                let position = WorldCell {
                    x: i32::try_from((native_x + 32768) / 2)
                        .map_err(|_| RangedError::Bound("flight x"))?,
                    y: i32::try_from((y + 32768).div_euclid(65536))
                        .map_err(|_| RangedError::Bound("flight y"))?,
                };
                remaining -= 1;
                if position == target {
                    remaining = 0;
                }
                let finished = remaining <= 0;
                let source_frame = if finished {
                    0
                } else if remaining == 1 {
                    2
                } else {
                    1
                };
                steps.push(RangedSample {
                    position,
                    heading,
                    source_frame,
                    finished,
                });
            }
        }
        Ok(Self {
            from,
            target,
            heading,
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
    pub fn finished_at(&self) -> Duration {
        Duration::from_nanos((self.steps.len() as u64 - 1) * STEP_NANOS)
    }
    pub fn sample(&self, elapsed: Duration) -> RangedSample {
        let index = (elapsed.as_nanos() / u128::from(STEP_NANOS)).min(self.steps.len() as u128 - 1)
            as usize;
        self.steps[index]
    }
    pub fn position(&self, elapsed: Duration) -> WorldCell {
        self.sample(elapsed).position
    }
}

// Literal4B9210 threshold branches in world units, no four-direction folding.
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

/// Index handles live only in load(). Missing MEMBER permits base fallback;
/// missing/broken archive or an ambiguous member never does. Name lookup uses
/// SharedArchive's canonical case-insensitive policy, not guessed case aliases.
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
            return Err(RangedError::Bound("index records"));
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
                .ok_or(RangedError::Bound("index records"))?;
            if self.records > MAX_INDEX_RECORDS {
                return Err(RangedError::Bound("index records"));
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

pub struct VisualSample<'a> {
    pub frame: &'a SpriteFrame,
    pub position: WorldCell,
    pub source_frame: usize,
    pub finished: bool,
    pub graphics_id: u32,
    pub sprite_name: &'a str,
    pub subframe: usize,
}

pub struct RangedAssets {
    animation: Animation,
    sprites: Vec<LoadedSprite>,
    catalog_sha256: [u8; 32],
    encoded: usize,
    rgba: usize,
}
impl RangedAssets {
    /// Worker-only. Fixed shared Text/list.spr and the qualified66's8 banks;
    /// every referenced SPR subframe is decoded before publishing any assets.
    /// This does NOT load bow bodies or their audio (existing actor assets own
    /// those), and #66 must contain no cue. Cancel/failure drops all handles.
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
        if size == 0 || size > MAX_CATALOG {
            return Err(RangedError::Bound("action catalog"));
        }
        active(cancel)?;
        let catalog_sha256 = context.sha256();
        let catalog = context.catalog();
        let animation = qualify(&catalog)?;
        let mut reader = SpriteReader::open(root, cancel)?;
        let mut sprites = reserve(8)?;
        let mut encoded = 0usize;
        let mut rgba = 0usize;
        for heading in 0..8 {
            active(cancel)?;
            let asset = animation.frame_asset(GRAPHICS_ID, 1, heading)?;
            let name = format!("{}-{}.spr", asset.graphics_id, asset.image_id);
            let archive = reader.archive_for(&name, cancel)?;
            let size = archive.entry(&name)?.size as usize;
            if size == 0 || size > MAX_ENTRY || size > MAX_ENCODED - encoded {
                return Err(RangedError::Bound("encoded sprites"));
            }
            let kind = archive.kind();
            active(cancel)?;
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            if bytes.len() != size {
                return Err(RangedError::Unsupported("SPR length changed"));
            }
            let pixels = (MAX_RGBA - rgba) / 4;
            if pixels == 0 {
                return Err(RangedError::Bound("decoded sprites"));
            }
            let decoded = decode_world_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_frames: 3,
                    max_blocks: 4096,
                    max_block_references: 16384,
                    max_frame_dimension: 512,
                    max_total_pixels: pixels,
                },
            )?;
            active(cancel)?;
            if decoded.frames.len() != 3 {
                return Err(RangedError::Unsupported("ordinary arrow subframes"));
            }
            let count = decoded
                .retained_rgba_bytes()
                .ok_or(RangedError::Bound("decoded sprites"))?;
            if count > MAX_RGBA - rgba {
                return Err(RangedError::Bound("decoded sprites"));
            }
            for source_frame in 0..3 {
                let bound = animation.frame_asset(GRAPHICS_ID, source_frame, heading)?;
                if bound.image_id != u32::from(heading)
                    || bound.subframe as usize >= decoded.frames.len()
                {
                    return Err(RangedError::Unsupported("ordinary arrow asset binding"));
                }
            }
            encoded += size;
            rgba += count;
            sprites.push(LoadedSprite {
                provenance: SpriteProvenance {
                    archive: kind,
                    name,
                    bytes: size as u32,
                    sha256: Sha256::digest(&bytes).into(),
                },
                decoded,
            });
        }
        drop(reader);
        active(cancel)?;
        Ok(Self {
            animation,
            sprites,
            catalog_sha256,
            encoded,
            rgba,
        })
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.catalog_sha256
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.sprites.iter().map(|sprite| &sprite.provenance)
    }
    /// Every decoded frame, including the never-drawn terminal subframe2.
    pub fn sprite_frames(&self, heading: u8) -> Result<&[SpriteFrame]> {
        Ok(&self.sprite(heading)?.decoded.frames)
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded
    }
    pub fn rgba_bytes(&self) -> usize {
        self.rgba
    }
    pub fn effect_duration(&self, path: &RangedPath) -> Duration {
        path.finished_at()
    }
    fn sprite(&self, heading: u8) -> Result<&LoadedSprite> {
        if heading > 7 {
            return Err(RangedError::InvalidInput("heading"));
        }
        self.sprites
            .get(usize::from(heading))
            .ok_or(RangedError::Unsupported("unloaded arrow heading"))
    }
    /// Finished samples expose terminal provenance only. They MUST NOT draw;
    /// draw_effect enforces this, including duration0 adjacent/overlapping shots.
    pub fn effect_frame(&self, path: &RangedPath, elapsed: Duration) -> Result<VisualSample<'_>> {
        let sample = path.sample(elapsed);
        let asset = self
            .animation
            .frame_asset(GRAPHICS_ID, sample.source_frame, sample.heading)?;
        let sprite = self.sprite(sample.heading)?;
        let frame = sprite
            .decoded
            .frames
            .get(asset.subframe as usize)
            .ok_or(RangedError::Unsupported("unloaded arrow subframe"))?;
        Ok(VisualSample {
            frame,
            position: sample.position,
            source_frame: sample.source_frame,
            finished: sample.finished,
            graphics_id: asset.graphics_id,
            sprite_name: &sprite.provenance.name,
            subframe: asset.subframe as usize,
        })
    }
    /// Current paint projection, frozen flight endpoint. Never uses a pending
    /// camera, actor interpolation, target lookup, collision or damage mutation.
    pub fn draw_effect(
        &self,
        canvas: &mut Canvas,
        path: &RangedPath,
        projection: ViewProjection,
        elapsed: Duration,
    ) -> Result<bool> {
        let sample = self.effect_frame(path, elapsed)?;
        if sample.finished {
            return Ok(false);
        }
        let anchor = projection
            .cell_to_pixel(sample.position)
            .ok_or(RangedError::Bound("projectile projection"))?;
        let x = anchor
            .x
            .checked_add(sample.frame.x_offset)
            .ok_or(RangedError::Bound("projected x"))?;
        let y = anchor
            .y
            .checked_add(sample.frame.y_offset)
            .ok_or(RangedError::Bound("projected y"))?;
        canvas
            .blit_rgba(
                &sample.frame.rgba,
                sample.frame.width,
                sample.frame.height,
                x,
                y,
            )
            .map_err(RangedError::Render)?;
        Ok(true)
    }
}

fn qualify(catalog: &SpriteActionCatalog) -> Result<Animation> {
    let sprite = catalog
        .sprites
        .get(&GRAPHICS_ID)
        .ok_or(RangedError::Unsupported("missing shared arrow66"))?;
    let resolved = catalog.resolve_action(GRAPHICS_ID, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != GRAPHICS_ID
        || resolved.definition_sprite_id != GRAPHICS_ID
        || sprite.image_count != Some(8)
        || sprite.actions.len() != 1
        || animation.directionality != Directionality::EightWay
        || animation.frames_per_second != 24
        || animation.frames.len() != 3
        || sprite.attributes.len() != 2
    {
        return Err(RangedError::Unsupported("unverified arrow66 definition"));
    }
    for (attribute, (key, body)) in sprite
        .attributes
        .iter()
        .zip([(102, b"9".as_slice()), (109, b"3 -1".as_slice())])
    {
        if attribute.key != ActionKey::new(key) || attribute.body_bytes != body {
            return Err(RangedError::Unsupported(
                "unverified arrow66 class or secondary effect",
            ));
        }
    }
    for (frame, subframe) in animation.frames.iter().zip([2, 0, 1]) {
        if frame.image_id != 0
            || frame.subframe != subframe
            || frame.duration_ticks != 1
            || !frame.events.is_empty()
        {
            return Err(RangedError::Unsupported(
                "unverified arrow66 frames or cues",
            ));
        }
    }
    Ok(animation.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str =
        "1 0 0\n#66 8 arrow 0.fly(1 3,0.2:1 0.0:1 0.1:1) 102.type(9) 109.effect(3 -1)\n";
    fn catalog(source: &str) -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(source.as_bytes(), Default::default()).unwrap()
    }
    fn cell(x: i32, y: i32) -> WorldCell {
        WorldCell { x, y }
    }
    fn fixture_assets() -> RangedAssets {
        let sprites = (0..8)
            .map(|heading| LoadedSprite {
                provenance: SpriteProvenance {
                    archive: SharedArchiveKind::Sprite,
                    name: format!("66-{heading}.spr"),
                    bytes: 0,
                    sha256: [0; 32],
                },
                decoded: DecodedSprite {
                    frames: (0..3)
                        .map(|subframe| SpriteFrame {
                            fragments: None,
                            x_offset: -1,
                            y_offset: -1,
                            width: 2,
                            height: 2,
                            frame_type: 0,
                            unknown1: 0,
                            unknown2: 0,
                            rgba: [heading * 20 + 1, subframe * 50 + 1, 220, 255].repeat(4),
                        })
                        .collect(),
                    has_palette: false,
                    block_table_metadata: 0,
                },
            })
            .collect();
        RangedAssets {
            animation: qualify(&catalog(SOURCE)).unwrap(),
            sprites,
            catalog_sha256: [0; 32],
            encoded: 0,
            rgba: 96,
        }
    }
    #[test]
    fn exact_definition_has_eight_unfolded_headings_three_subframes_and_no_cues() {
        let animation = qualify(&catalog(SOURCE)).unwrap();
        for heading in 0..8 {
            for (source, subframe) in [2, 0, 1].into_iter().enumerate() {
                let asset = animation.frame_asset(66, source, heading).unwrap();
                assert_eq!(
                    (asset.graphics_id, asset.image_id, asset.subframe),
                    (66, u32::from(heading), subframe)
                );
            }
        }
        for bad in [
            SOURCE.replace("#66 8", "#66 4"),
            SOURCE.replace("1 3,", "0 3,"),
            SOURCE.replace("3 -1", "3 219"),
            SOURCE.replace("3 -1", "4 -1"),
            SOURCE.replace("type(9)", "type(8)"),
            SOURCE.replace("0.2:1", "0.2:2"),
            SOURCE.replace("0.0:1", "0.0:1[13"),
            SOURCE.replace("0.0:1", "0.0:1!"),
            SOURCE.replace("0.1:1", "1.1:1"),
            SOURCE.replace("0.1:1", "0.2:1"),
            SOURCE.replace("102.type", "110.fps(25) 102.type"),
        ] {
            assert!(
                qualify(&catalog(&bad)).is_err(),
                "accepted changed arrow definition"
            );
        }
    }
    #[test]
    fn immediate_step_30ms_cadence_and_native_counter_choose_frames_not_24fps() {
        let from = cell(32724, 32784);
        let vertical = RangedPath::new(from, cell(from.x, from.y + 3)).unwrap();
        assert_eq!(vertical.finished_at(), Duration::from_millis(60));
        assert_eq!(
            vertical.sample(Duration::ZERO),
            RangedSample {
                position: cell(from.x, from.y + 1),
                heading: 4,
                source_frame: 1,
                finished: false
            }
        );
        assert_eq!(
            vertical.sample(Duration::from_nanos(29_999_999)),
            vertical.sample(Duration::ZERO)
        );
        assert_eq!(vertical.sample(Duration::from_millis(30)).source_frame, 2);
        assert_eq!(
            vertical.sample(Duration::from_millis(60)),
            RangedSample {
                position: vertical.target(),
                heading: 4,
                source_frame: 0,
                finished: true
            }
        );
        let horizontal = RangedPath::new(from, cell(from.x + 3, from.y)).unwrap();
        assert_eq!(
            horizontal
                .steps
                .iter()
                .map(|sample| sample.source_frame)
                .collect::<Vec<_>>(),
            [1, 1, 0]
        );
        assert_eq!(horizontal.finished_at(), Duration::from_millis(60));
    }
    #[test]
    fn native_rounding_clears_x_low_bit_after_rounding_and_all_directions_stay_distinct() {
        let from = cell(32724, 32784);
        let path = RangedPath::new(from, cell(from.x + 1, from.y + 5)).unwrap();
        assert_eq!(
            path.steps
                .iter()
                .map(|sample| sample.position.x - from.x)
                .collect::<Vec<_>>(),
            [0, 0, 0, 1, 1]
        );
        for (heading, (dx, dy)) in [
            (0, (0, -3)),
            (1, (3, -3)),
            (2, (3, 0)),
            (3, (3, 3)),
            (4, (0, 3)),
            (5, (-3, 3)),
            (6, (-3, 0)),
            (7, (-3, -3)),
        ] {
            let path = RangedPath::new(from, cell(from.x + dx, from.y + dy)).unwrap();
            assert_eq!(path.heading(), heading);
            assert_eq!(path.sample(Duration::MAX).position, path.target());
        }
    }
    #[test]
    fn adjacent_and_same_cell_have_no_visible_terminal_dwell_or_elapsed_replay() {
        let from = cell(32724, 32784);
        for dx in -1..=1 {
            for dy in -1..=1 {
                let path = RangedPath::new(from, cell(from.x + dx, from.y + dy)).unwrap();
                assert_eq!(path.finished_at(), Duration::ZERO);
                assert!(path.sample(Duration::ZERO).finished);
                assert_eq!(path.sample(Duration::ZERO), path.sample(Duration::MAX));
            }
        }
    }
    #[test]
    fn bounded_launch_allocation_and_native_coordinate_domain_are_explicit() {
        let from = cell(32724, 32784);
        let longest = RangedPath::new(from, cell(from.x + 256, from.y)).unwrap();
        assert!(longest.steps.len() <= 512);
        assert_eq!(longest.finished_at(), Duration::from_millis(7650));
        assert!(longest.sample(Duration::MAX).finished);
        for target in [
            cell(from.x + 257, from.y),
            cell(16383, 1),
            cell(49152, 1),
            cell(32724, -1),
            cell(32724, 65536),
            cell(i32::MAX, i32::MIN),
        ] {
            assert!(matches!(
                RangedPath::new(from, target),
                Err(RangedError::Bound(_))
            ));
        }
        for from in [cell(16384, 0), cell(49151, 65535)] {
            assert!(
                RangedPath::new(from, from)
                    .unwrap()
                    .sample(Duration::ZERO)
                    .finished
            );
        }
    }
    // Separate32-bit instruction transcription checks the wider implementation
    // against native SHR/wrapping-add and signedIDIV, not a second lerp formula.
    fn instruction_reference(from: WorldCell, target: WorldCell) -> Vec<(WorldCell, usize, bool)> {
        let nx = 2 * from.x - 32768;
        let tx = 2 * target.x - 32768;
        let dx = (tx - nx) / 2;
        let dy = target.y - from.y;
        let divisor = dx.abs().max(dy.abs());
        let vx = if divisor == 0 {
            0
        } else {
            ((dx << 16) / divisor) * 2
        };
        let vy = if divisor == 0 {
            0
        } else {
            (dy << 16) / divisor
        };
        let mut x = (nx as u32) << 16;
        let mut y = (from.y as u32) << 16;
        let mut remaining = (tx - nx).abs().max(dy.abs());
        let mut result = Vec::new();
        loop {
            x = x.wrapping_add(vx as u32);
            y = y.wrapping_add(vy as u32);
            let px = (x.wrapping_add(0x8000) >> 16) & !1;
            let py = y.wrapping_add(0x8000) >> 16;
            let position = cell((px as i32 + 32768) / 2, py as i32);
            remaining -= 1;
            if position == target {
                remaining = 0;
            }
            let finished = remaining <= 0;
            let loaded_index = if remaining > 1 {
                0
            } else if remaining == 1 {
                1
            } else {
                2
            };
            result.push((position, (loaded_index + 1) % 3, finished));
            if finished {
                return result;
            }
            assert!(result.len() <= 512);
        }
    }
    #[test]
    fn bounded_paths_match_independent_native_word_instruction_transcription() {
        for from in [cell(32724, 32784), cell(16392, 8), cell(49143, 65527)] {
            for dx in -8..=8 {
                for dy in -8..=8 {
                    let target = cell(from.x + dx, from.y + dy);
                    let actual = RangedPath::new(from, target).unwrap();
                    let samples: Vec<_> = actual
                        .steps
                        .iter()
                        .map(|sample| (sample.position, sample.source_frame, sample.finished))
                        .collect();
                    assert_eq!(
                        samples,
                        instruction_reference(from, target),
                        "delta {dx},{dy}"
                    );
                }
            }
        }
        for dx in [-256, -255, -129, -1, 0, 1, 129, 255, 256] {
            for dy in [-256, -255, -129, -1, 0, 1, 129, 255, 256] {
                let from = cell(32724, 32784);
                let target = cell(from.x + dx, from.y + dy);
                let actual = RangedPath::new(from, target).unwrap();
                assert_eq!(
                    actual
                        .steps
                        .iter()
                        .map(|sample| (sample.position, sample.source_frame, sample.finished))
                        .collect::<Vec<_>>(),
                    instruction_reference(from, target)
                );
            }
        }
    }
    #[test]
    fn synthetic_pixels_use_position_and_heading_but_never_draw_terminal_or_impact() {
        let assets = fixture_assets();
        let from = cell(32724, 32784);
        let projection = ViewProjection {
            anchor_cell: from,
            anchor_pixel: super::super::world_map::PixelPoint { x: 200, y: 200 },
        };
        for (heading, (dx, dy)) in [
            (0, (0, -3)),
            (1, (3, -3)),
            (2, (3, 0)),
            (3, (3, 3)),
            (4, (0, 3)),
            (5, (-3, 3)),
            (6, (-3, 0)),
            (7, (-3, -3)),
        ] {
            let path = RangedPath::new(from, cell(from.x + dx, from.y + dy)).unwrap();
            let sample = assets.effect_frame(&path, Duration::ZERO).unwrap();
            assert_eq!(sample.sprite_name, format!("66-{heading}.spr"));
            assert_eq!(
                (sample.source_frame, sample.subframe, sample.finished),
                (1, 0, false)
            );
            let mut canvas = Canvas::new(400, 400).unwrap();
            let blank = canvas.rgba().to_vec();
            assert!(assets
                .draw_effect(&mut canvas, &path, projection, Duration::ZERO)
                .unwrap());
            assert_ne!(canvas.rgba(), blank);
            canvas.clear([0, 0, 0]);
            assert!(!assets
                .draw_effect(&mut canvas, &path, projection, path.finished_at())
                .unwrap());
            assert_eq!(canvas.rgba(), blank);
            assert!(!assets
                .draw_effect(&mut canvas, &path, projection, Duration::MAX)
                .unwrap());
            assert_eq!(canvas.rgba(), blank);
        }
        assert!(assets.sprite_frames(8).is_err());
    }
    #[test]
    fn cancelled_loader_does_not_open_even_an_absent_root() {
        assert!(matches!(
            RangedAssets::load(Path::new("absent-arrow-root"), &AtomicBool::new(true)),
            Err(RangedError::Cancelled)
        ));
    }
}
