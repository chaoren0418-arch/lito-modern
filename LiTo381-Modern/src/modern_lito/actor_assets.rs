//! Preload one verified shared list.spr action into an owned rendering snapshot.
//!
//! Graphics identity comes from the requested catalog record (LinkedId in
//! PakViewer's SprListEntry.SpriteId), never from the numeric action id. Each
//! source frame names graphics-(image + heading).spr and an exact SPR subframe.
//! The archive must be explicit or unique across ALL seventeen shared Sprite
//! archives; collisions, absent images and absent subframes never pick a fallback.
//! This loads pixels and presentation cues, not sound/overlay/effect assets or
//! authoritative combat logic. It does not consult LT archives or native code.

use std::{collections::BTreeMap, fmt, path::Path};

use super::animation::{AnimationClock, AnimationError, PlaybackLimits};
use super::resources::{ResourceError, SharedArchive, SharedArchiveKind};
use super::sprite::{
    decode_classic_spr_with_limits, DecodedSprite, SpriteError, SpriteFrame, SpriteLimits,
};
use super::sprite_actions::{
    ActionError, ActionKey, Directionality, FrameAsset, SpriteActionCatalog,
};

const MAX_INDEX_RECORDS: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpriteArchiveSelection {
    Explicit(SharedArchiveKind),
    /// Requires Sprite and every Sprite00..15 index/PAK pair to be available.
    /// A missing shard is an incomplete collision check, not a reason to use
    /// whichever archive happened to open first.
    UniqueAcrossShared,
}

#[derive(Clone, Copy, Debug)]
pub struct ActorAssetLimits {
    pub max_sprite_entries: usize,
    pub max_entry_bytes: usize,
    /// Sum of unique source payload sizes, checked BEFORE the first read.
    pub max_total_encoded_bytes: usize,
    /// All decoded subframes in retained unique SPR entries, not only those
    /// displayed by the action. One RGBA pixel consumes four cache bytes.
    pub max_total_pixels: usize,
    pub playback: PlaybackLimits,
}

impl Default for ActorAssetLimits {
    fn default() -> Self {
        Self {
            max_sprite_entries: 128,
            max_entry_bytes: 16 * 1024 * 1024,
            max_total_encoded_bytes: 64 * 1024 * 1024,
            max_total_pixels: 16 * 1024 * 1024,
            playback: PlaybackLimits::default(),
        }
    }
}

impl ActorAssetLimits {
    fn validate(self) -> Result<Self, ActorAssetError> {
        if self.max_sprite_entries == 0
            || self.max_sprite_entries > 4096
            || self.max_entry_bytes == 0
            || self.max_entry_bytes > 64 * 1024 * 1024
            || self.max_total_encoded_bytes == 0
            || self.max_total_encoded_bytes > 256 * 1024 * 1024
            || self.max_total_pixels == 0
            || self.max_total_pixels > 64 * 1024 * 1024
        {
            return Err(ActorAssetError::InvalidLimits);
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActorActionRequest {
    pub sprite_id: u32,
    pub key: ActionKey,
    pub heading: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActorActionProvenance {
    pub requested_sprite_id: u32,
    pub definition_sprite_id: u32,
    pub graphics_id: u32,
    pub key: ActionKey,
    pub heading: u8,
}

#[derive(Debug)]
pub enum ActorAssetError {
    InvalidLimits,
    InvalidArchive(SharedArchiveKind),
    InvalidHeading(u8),
    LimitExceeded(&'static str),
    Action(ActionError),
    Clock(AnimationError),
    Resource {
        archive: SharedArchiveKind,
        source: ResourceError,
    },
    Sprite {
        archive: SharedArchiveKind,
        name: String,
        source: SpriteError,
    },
    MissingEntry(String),
    Collision {
        name: String,
        archives: Vec<SharedArchiveKind>,
    },
    UnverifiedGraphicsInheritance {
        requested: u32,
        definition: u32,
    },
    ImageIdOverflow,
    MissingSubframe {
        name: String,
        action_frame: usize,
        subframe: u32,
        available: usize,
    },
    ActionFrameOutOfRange(usize),
    AllocationFailed,
}

impl fmt::Display for ActorAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits => write!(f, "invalid bounded actor-asset limits"),
            Self::InvalidArchive(kind) => write!(
                f,
                "actor pixels require a shared Sprite archive, not {kind:?}"
            ),
            Self::InvalidHeading(heading) => write!(f, "actor heading {heading} is outside 0..7"),
            Self::LimitExceeded(what) => write!(f, "actor assets exceed {what} budget"),
            Self::Action(error) => write!(f, "actor action: {error}"),
            Self::Clock(error) => write!(f, "actor playback: {error}"),
            Self::Resource { archive, source } => write!(f, "actor {archive:?}: {source}"),
            Self::Sprite {
                archive,
                name,
                source,
            } => write!(f, "actor {archive:?}/{name}: {source}"),
            Self::MissingEntry(name) => write!(
                f,
                "actor SPR absent from selected shared archive scope: {name}"
            ),
            Self::Collision { name, archives } => write!(
                f,
                "actor SPR {name} collides in {archives:?}; select an explicit archive"
            ),
            Self::UnverifiedGraphicsInheritance {
                requested,
                definition,
            } => write!(
                f,
                "action alias graphics ownership unverified: requested bank {requested}, definition bank {definition}"
            ),
            Self::ImageIdOverflow => write!(f, "directional actor image id overflow"),
            Self::MissingSubframe {
                name,
                action_frame,
                subframe,
                available,
            } => write!(
                f,
                "actor action frame {action_frame} needs {name} subframe {subframe}, but only {available} exist"
            ),
            Self::ActionFrameOutOfRange(index) => {
                write!(f, "actor source action frame {index} out of range")
            }
            Self::AllocationFailed => write!(f, "unable to allocate bounded actor assets"),
        }
    }
}
impl std::error::Error for ActorAssetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Action(error) => Some(error),
            Self::Clock(error) => Some(error),
            Self::Resource { source, .. } => Some(source),
            Self::Sprite { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn bounded_vec<T>(count: usize) -> Result<Vec<T>, ActorAssetError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_| ActorAssetError::AllocationFailed)?;
    Ok(output)
}

/// Reusable loader-side handles. A completed PreloadedActorAction borrows
/// nothing from this object, so it may be dropped before handing assets to UI.
pub struct SharedSpriteArchives {
    archives: Vec<SharedArchive>,
}

impl SharedSpriteArchives {
    pub fn open(
        root: impl AsRef<Path>,
        selection: SpriteArchiveSelection,
    ) -> Result<Self, ActorAssetError> {
        let kinds = match selection {
            SpriteArchiveSelection::Explicit(
                kind @ (SharedArchiveKind::Sprite | SharedArchiveKind::SpriteShard(0..=15)),
            ) => vec![kind],
            SpriteArchiveSelection::Explicit(kind) => {
                return Err(ActorAssetError::InvalidArchive(kind));
            }
            SpriteArchiveSelection::UniqueAcrossShared => {
                let mut kinds = vec![SharedArchiveKind::Sprite];
                kinds.extend((0..16).map(SharedArchiveKind::SpriteShard));
                kinds
            }
        };
        let mut archives = bounded_vec(kinds.len())?;
        let mut records = 0usize;
        for kind in kinds {
            let archive =
                SharedArchive::open(&root, kind).map_err(|source| ActorAssetError::Resource {
                    archive: kind,
                    source,
                })?;
            records = records
                .checked_add(archive.len())
                .ok_or(ActorAssetError::LimitExceeded("index records"))?;
            if records > MAX_INDEX_RECORDS {
                return Err(ActorAssetError::LimitExceeded("index records"));
            }
            archives.push(archive);
        }
        Ok(Self { archives })
    }

    fn locate(&self, name: &str) -> Result<LocatedEntry, ActorAssetError> {
        let mut matches = bounded_vec(self.archives.len())?;
        for (index, archive) in self.archives.iter().enumerate() {
            match archive.entry(name) {
                Ok(entry) => matches.push((index, archive.kind(), entry.size as usize)),
                Err(ResourceError::MissingEntry(_)) => {}
                Err(source) => {
                    return Err(ActorAssetError::Resource {
                        archive: archive.kind(),
                        source,
                    });
                }
            }
        }
        match matches.as_slice() {
            [] => Err(ActorAssetError::MissingEntry(name.to_owned())),
            &[(archive_index, archive, size)] => Ok(LocatedEntry {
                archive_index,
                archive,
                size,
                name: name.to_owned(),
            }),
            _ => Err(ActorAssetError::Collision {
                name: name.to_owned(),
                archives: matches.iter().map(|value| value.1).collect(),
            }),
        }
    }
}

struct LocatedEntry {
    archive_index: usize,
    archive: SharedArchiveKind,
    size: usize,
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundActionFrame {
    pub asset: FrameAsset,
    /// Index into this snapshot's unique decoded SPR entries, not a catalog id.
    pub sprite_entry_index: usize,
}

pub struct LoadedActorSprite {
    pub archive: SharedArchiveKind,
    pub name: String,
    pub decoded: DecodedSprite,
}

pub struct ActionFrameAssets {
    bindings: Vec<BoundActionFrame>,
    sprites: Vec<LoadedActorSprite>,
    decoded_pixels: usize,
    encoded_bytes: usize,
}

impl ActionFrameAssets {
    pub fn bindings(&self) -> &[BoundActionFrame] {
        &self.bindings
    }
    pub fn sprites(&self) -> &[LoadedActorSprite] {
        &self.sprites
    }
    pub fn decoded_pixels(&self) -> usize {
        self.decoded_pixels
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    /// Takes a SOURCE action-frame index (e.g. AnimationSample.frame_index),
    /// not an image id. Every sprite/subframe pair was validated during load.
    pub fn frame(&self, source_frame: usize) -> Result<&SpriteFrame, ActorAssetError> {
        let binding = self
            .bindings
            .get(source_frame)
            .ok_or(ActorAssetError::ActionFrameOutOfRange(source_frame))?;
        Ok(&self.sprites[binding.sprite_entry_index].decoded.frames
            [binding.asset.subframe as usize])
    }
}

pub struct PreloadedActorAction {
    pub provenance: ActorActionProvenance,
    /// Separate owned fields allow sampling the clock and reading the pixels
    /// without a self-referential structure or a whole-snapshot borrow.
    pub clock: AnimationClock,
    pub assets: ActionFrameAssets,
}

pub fn preload_action(
    catalog: &SpriteActionCatalog,
    archives: &mut SharedSpriteArchives,
    request: ActorActionRequest,
    limits: ActorAssetLimits,
) -> Result<PreloadedActorAction, ActorAssetError> {
    let limits = limits.validate()?;
    if request.heading > 7 {
        return Err(ActorAssetError::InvalidHeading(request.heading));
    }
    let resolved = catalog
        .resolve_action(request.sprite_id, request.key)
        .map_err(ActorAssetError::Action)?;
    // Resolution has checked both headers, presence and quarantine. Catalog
    // graphics aliases are direct; do not recursively look up a graphics id.
    let graphics_id = catalog.sprites[&request.sprite_id].graphics_id;
    let definition_graphics = catalog.sprites[&resolved.definition_sprite_id].graphics_id;
    if resolved.definition_sprite_id != request.sprite_id && graphics_id != definition_graphics {
        return Err(ActorAssetError::UnverifiedGraphicsInheritance {
            requested: graphics_id,
            definition: definition_graphics,
        });
    }
    let clock = AnimationClock::with_limits(resolved.animation, limits.playback)
        .map_err(ActorAssetError::Clock)?;
    let animation = clock.animation();
    let direction = if animation.directionality == Directionality::EightWay {
        u32::from(request.heading)
    } else {
        0
    };
    let mut bindings = bounded_vec(animation.frames.len())?;
    let mut locations = bounded_vec(limits.max_sprite_entries.min(animation.frames.len()))?;
    let mut image_ids = BTreeMap::new();
    let mut encoded_bytes = 0usize;
    // Plan and validate every unique source payload before reading any. Do
    // not call Animation.frame_asset for each frame: its semantics scan would
    // turn a large action into quadratic work; the owned clock validated once.
    for frame in &animation.frames {
        let image_id = frame
            .image_id
            .checked_add(direction)
            .ok_or(ActorAssetError::ImageIdOverflow)?;
        let asset = FrameAsset {
            graphics_id,
            image_id,
            subframe: frame.subframe,
        };
        let sprite_entry_index = match image_ids.get(&image_id) {
            Some(&index) => index,
            None => {
                if locations.len() >= limits.max_sprite_entries {
                    return Err(ActorAssetError::LimitExceeded("unique SPR entries"));
                }
                let location = archives.locate(&asset.file_name())?;
                if location.size > limits.max_entry_bytes {
                    return Err(ActorAssetError::LimitExceeded("single encoded SPR bytes"));
                }
                encoded_bytes = encoded_bytes
                    .checked_add(location.size)
                    .ok_or(ActorAssetError::LimitExceeded("total encoded bytes"))?;
                if encoded_bytes > limits.max_total_encoded_bytes {
                    return Err(ActorAssetError::LimitExceeded("total encoded bytes"));
                }
                let index = locations.len();
                locations.push(location);
                image_ids.insert(image_id, index);
                index
            }
        };
        bindings.push(BoundActionFrame {
            asset,
            sprite_entry_index,
        });
    }
    let mut sprites = bounded_vec(locations.len())?;
    let mut decoded_pixels = 0usize;
    for location in locations {
        let remaining = limits.max_total_pixels - decoded_pixels;
        if remaining == 0 {
            return Err(ActorAssetError::LimitExceeded("total decoded pixels"));
        }
        let bytes = archives.archives[location.archive_index]
            .read_entry(&location.name)
            .map_err(|source| ActorAssetError::Resource {
                archive: location.archive,
                source,
            })?;
        let decoded = decode_classic_spr_with_limits(
            &bytes,
            SpriteLimits {
                max_input_bytes: limits.max_entry_bytes,
                max_total_pixels: remaining,
                ..SpriteLimits::default()
            },
        )
        .map_err(|source| ActorAssetError::Sprite {
            archive: location.archive,
            name: location.name.clone(),
            source,
        })?;
        let pixels = decoded
            .frames
            .iter()
            .try_fold(0usize, |sum, frame| sum.checked_add(frame.rgba.len() / 4))
            .ok_or(ActorAssetError::LimitExceeded("total decoded pixels"))?;
        decoded_pixels = decoded_pixels
            .checked_add(pixels)
            .ok_or(ActorAssetError::LimitExceeded("total decoded pixels"))?;
        if decoded_pixels > limits.max_total_pixels {
            return Err(ActorAssetError::LimitExceeded("total decoded pixels"));
        }
        sprites.push(LoadedActorSprite {
            archive: location.archive,
            name: location.name,
            decoded,
        });
    }
    for (action_frame, binding) in bindings.iter().enumerate() {
        let sprite = &sprites[binding.sprite_entry_index];
        if binding.asset.subframe as usize >= sprite.decoded.frames.len() {
            return Err(ActorAssetError::MissingSubframe {
                name: sprite.name.clone(),
                action_frame,
                subframe: binding.asset.subframe,
                available: sprite.decoded.frames.len(),
            });
        }
    }
    Ok(PreloadedActorAction {
        provenance: ActorActionProvenance {
            requested_sprite_id: request.sprite_id,
            definition_sprite_id: resolved.definition_sprite_id,
            graphics_id,
            key: request.key,
            heading: request.heading,
        },
        clock,
        assets: ActionFrameAssets {
            bindings,
            sprites,
            decoded_pixels,
            encoded_bytes,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::sprite_actions::{ActionLimits, FrameEvent};
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
    struct Fixture {
        path: PathBuf,
        parent: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/actor-assets-tests");
            std::fs::create_dir_all(&parent).unwrap();
            let parent = std::fs::canonicalize(parent).unwrap();
            for _ in 0..64 {
                let path = parent.join(format!(
                    "{}-{}",
                    std::process::id(),
                    NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
                ));
                match std::fs::create_dir(&path) {
                    Ok(()) => return Self { path, parent },
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(e) => panic!("fixture directory: {e}"),
                }
            }
            panic!("fixture unique directory exhausted");
        }
        fn archive(&self, kind: SharedArchiveKind, entries: &[(&str, Vec<u8>)]) {
            let (index, pak) = kind.file_names().unwrap();
            let mut index_bytes = (entries.len() as u32).to_le_bytes().to_vec();
            let mut body = Vec::new();
            for (name, bytes) in entries {
                assert!(name.len() <= 20);
                index_bytes.extend((body.len() as u32).to_le_bytes());
                let mut field = [0; 20];
                field[..name.len()].copy_from_slice(name.as_bytes());
                index_bytes.extend(field);
                index_bytes.extend((bytes.len() as u32).to_le_bytes());
                body.extend(bytes);
            }
            std::fs::write(self.path.join(index), index_bytes).unwrap();
            std::fs::write(self.path.join(pak), body).unwrap();
        }
        fn complete_empty_shards(&self) {
            for shard in 0..16 {
                self.archive(SharedArchiveKind::SpriteShard(shard), &[]);
            }
        }
        fn explicit(&self) -> SharedSpriteArchives {
            SharedSpriteArchives::open(
                &self.path,
                SpriteArchiveSelection::Explicit(SharedArchiveKind::Sprite),
            )
            .unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // Only our uniquely create_dir-owned exact child may be removed.
            // Refuse any replaced/symlinked/out-of-target fixture path.
            if let Ok(actual) = std::fs::canonicalize(&self.path) {
                if actual == self.path && actual.parent() == Some(self.parent.as_path()) {
                    let _ = std::fs::remove_dir_all(&actual);
                }
            }
        }
    }
    fn spr(color: u16) -> Vec<u8> {
        let mut data = vec![1];
        for value in [0i16, 0, 23, 23] {
            data.extend(value.to_le_bytes());
        }
        data.extend([0; 4]);
        data.extend(1u16.to_le_bytes());
        data.extend([0, 0, 18]);
        data.extend(0u16.to_le_bytes());
        let mut block = vec![0, 0, 1, 1, 1, 0, 1];
        block.extend(color.to_le_bytes());
        data.extend(1u32.to_le_bytes());
        data.extend(0u32.to_le_bytes());
        data.extend((block.len() as u32).to_le_bytes());
        data.extend(block);
        data
    }
    fn catalog(body: &[u8]) -> SpriteActionCatalog {
        SpriteActionCatalog::parse(body, ActionLimits::default()).unwrap()
    }
    fn request(heading: u8) -> ActorActionRequest {
        ActorActionRequest {
            sprite_id: 0,
            key: ActionKey::new(1),
            heading,
        }
    }

    #[test]
    fn graphics_heading_subframe_and_owned_snapshot_are_bound_exactly() {
        let fixture = Fixture::new();
        fixture.archive(SharedArchiveKind::Sprite, &[("42-20.spr", spr(0x7c00))]);
        let mut archives = fixture.explicit();
        let source = catalog(b"#0 1=42\n1.attack(1 2,16.0:0 16.0:3!)\n");
        let mut loaded = preload_action(
            &source,
            &mut archives,
            request(4),
            ActorAssetLimits::default(),
        )
        .unwrap();
        drop(source);
        drop(archives);
        assert_eq!(loaded.provenance.graphics_id, 42);
        assert_eq!(loaded.assets.sprites().len(), 1);
        assert_eq!(loaded.assets.bindings().len(), 2);
        assert_eq!(loaded.assets.sprites()[0].name, "42-20.spr");
        assert_eq!(
            &loaded.assets.frame(1).unwrap().rgba[..4],
            &[255, 0, 0, 255]
        );
        let sampled = loaded.clock.sample(Duration::ZERO).unwrap();
        assert_eq!(sampled.frame_index, 1);
        assert_eq!(sampled.events[0].event, &FrameEvent::Hit);
        assert_eq!(loaded.assets.frame(sampled.frame_index).unwrap().width, 24);
        assert!(matches!(
            loaded.assets.frame(2),
            Err(ActorAssetError::ActionFrameOutOfRange(2))
        ));
    }

    #[test]
    fn same_name_collision_is_rejected_even_if_payloads_are_identical() {
        let fixture = Fixture::new();
        fixture.complete_empty_shards();
        fixture.archive(SharedArchiveKind::Sprite, &[("42-0.spr", spr(31))]);
        fixture.archive(SharedArchiveKind::SpriteShard(15), &[("42-0.spr", spr(31))]);
        let source = catalog(b"#0=42\n1.idle(0 1,0.0:2)\n");
        let mut archives =
            SharedSpriteArchives::open(&fixture.path, SpriteArchiveSelection::UniqueAcrossShared)
                .unwrap();
        assert!(
            matches!(preload_action(&source, &mut archives, request(7), ActorAssetLimits::default()), Err(ActorAssetError::Collision { archives, .. }) if archives == vec![SharedArchiveKind::Sprite, SharedArchiveKind::SpriteShard(15)])
        );
        let mut explicit = fixture.explicit();
        let loaded = preload_action(
            &source,
            &mut explicit,
            request(7),
            ActorAssetLimits::default(),
        )
        .unwrap();
        assert_eq!(loaded.assets.sprites()[0].name, "42-0.spr");
    }

    #[test]
    fn incomplete_shared_scope_wrong_archive_and_unknown_actions_fail_closed() {
        let fixture = Fixture::new();
        fixture.archive(SharedArchiveKind::Sprite, &[]);
        assert!(SharedSpriteArchives::open(
            &fixture.path,
            SpriteArchiveSelection::UniqueAcrossShared
        )
        .is_err());
        assert!(matches!(
            SharedSpriteArchives::open(
                &fixture.path,
                SpriteArchiveSelection::Explicit(SharedArchiveKind::Tile)
            ),
            Err(ActorAssetError::InvalidArchive(_))
        ));
        let source = catalog(b"#0=42\n1.idle(0 1,0.0:2>)\n");
        assert!(matches!(
            preload_action(
                &source,
                &mut fixture.explicit(),
                request(0),
                ActorAssetLimits::default()
            ),
            Err(ActorAssetError::Action(_))
        ));
    }

    #[test]
    fn missing_spr_and_subframe_including_setup_are_not_substituted() {
        let fixture = Fixture::new();
        fixture.archive(SharedArchiveKind::Sprite, &[("42-0.spr", spr(31))]);
        let mut archives = fixture.explicit();
        let missing = catalog(b"#0=42\n1.idle(0 1,8.0:2)\n");
        assert!(matches!(
            preload_action(
                &missing,
                &mut archives,
                request(0),
                ActorAssetLimits::default()
            ),
            Err(ActorAssetError::MissingEntry(_))
        ));
        let missing_frame = catalog(b"#0=42\n1.idle(0 2,0.1:0 0.0:2)\n");
        assert!(matches!(
            preload_action(
                &missing_frame,
                &mut archives,
                request(0),
                ActorAssetLimits::default()
            ),
            Err(ActorAssetError::MissingSubframe {
                action_frame: 0,
                subframe: 1,
                available: 1,
                ..
            })
        ));
    }

    #[test]
    fn entry_encoded_pixel_and_playback_budgets_are_enforced() {
        let fixture = Fixture::new();
        fixture.archive(
            SharedArchiveKind::Sprite,
            &[("42-0.spr", spr(31)), ("42-8.spr", spr(0x7c00))],
        );
        let source = catalog(b"#0=42\n1.idle(0 2,0.0:1 8.0:2)\n");
        let mut archives = fixture.explicit();
        for limits in [
            ActorAssetLimits {
                max_sprite_entries: 1,
                ..Default::default()
            },
            ActorAssetLimits {
                max_entry_bytes: 1,
                ..Default::default()
            },
            ActorAssetLimits {
                max_total_encoded_bytes: spr(31).len(),
                ..Default::default()
            },
            ActorAssetLimits {
                max_total_pixels: 575,
                ..Default::default()
            },
            ActorAssetLimits {
                max_total_pixels: 1000,
                ..Default::default()
            },
            ActorAssetLimits {
                playback: PlaybackLimits {
                    max_frames: 1,
                    ..Default::default()
                },
                ..Default::default()
            },
        ] {
            assert!(preload_action(&source, &mut archives, request(0), limits).is_err());
        }
        let loaded = preload_action(
            &source,
            &mut archives,
            request(0),
            ActorAssetLimits {
                max_total_pixels: 1152,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(loaded.assets.decoded_pixels(), 1152);
        assert_eq!(loaded.assets.encoded_bytes(), spr(31).len() * 2);
        assert!(matches!(
            preload_action(
                &source,
                &mut archives,
                request(8),
                ActorAssetLimits::default()
            ),
            Err(ActorAssetError::InvalidHeading(8))
        ));
    }

    #[test]
    fn alias_with_different_graphics_and_direction_overflow_are_not_guessed() {
        let fixture = Fixture::new();
        fixture.archive(SharedArchiveKind::Sprite, &[]);
        let source = catalog(b"#0=42\n1=1\n#1=99\n1.idle(0 1,0.0:2)\n");
        assert!(matches!(
            preload_action(
                &source,
                &mut fixture.explicit(),
                request(0),
                ActorAssetLimits::default()
            ),
            Err(ActorAssetError::UnverifiedGraphicsInheritance {
                requested: 42,
                definition: 99
            })
        ));
        let source = catalog(b"#0=42\n1.idle(1 1,4294967295.0:2)\n");
        assert!(matches!(
            preload_action(
                &source,
                &mut fixture.explicit(),
                request(1),
                ActorAssetLimits::default()
            ),
            Err(ActorAssetError::ImageIdOverflow)
        ));
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; explicit Sprite eight-heading goldens and genuine shared collision rejection"]
    fn real_explicit_sprite_bank_binds_eight_headings_and_unique_scope_rejects_collision() {
        use sha2::{Digest, Sha256};
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared resource root");
        let mut text = SharedArchive::open(&root, SharedArchiveKind::Text).unwrap();
        let bytes = text.read_entry("list.spr").unwrap();
        let source =
            SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
        let mut unique =
            SharedSpriteArchives::open(&root, SpriteArchiveSelection::UniqueAcrossShared).unwrap();
        // The inspected scene's heading zero is unambiguous. Heading four
        // is NOT: 3225-4.spr exists with different bytes in Sprite/Sprite00.
        // Never turn the C# tool's unsorted Directory.GetFiles + dictionary
        // last-write-wins into an alleged original-client archive priority.
        for key in [1, 3] {
            preload_action(
                &source,
                &mut unique,
                ActorActionRequest {
                    sprite_id: 0,
                    key: ActionKey::new(key),
                    heading: 0,
                },
                ActorAssetLimits::default(),
            )
            .unwrap();
        }
        assert!(matches!(
            preload_action(&source, &mut unique, request(4), ActorAssetLimits::default()),
            Err(ActorAssetError::Collision { name, archives })
                if name == "3225-4.spr"
                    && archives == vec![SharedArchiveKind::Sprite, SharedArchiveKind::SpriteShard(0)]
        ));
        for (kind, size, checksum) in [
            (
                SharedArchiveKind::Sprite,
                5313,
                "DAD0417AC727D179022A3D39039CC8EB4107072E0479C004045D97842DC192DA",
            ),
            (
                SharedArchiveKind::SpriteShard(0),
                7649,
                "2BD2B24CF35E621696E44A23BA30251831FBBBD478CE3DF73428E9A7A4A045C9",
            ),
        ] {
            let bytes = unique
                .archives
                .iter_mut()
                .find(|archive| archive.kind() == kind)
                .unwrap()
                .read_entry("3225-4.spr")
                .unwrap();
            assert_eq!(bytes.len(), size);
            assert_eq!(format!("{:X}", Sha256::digest(&bytes)), checksum);
        }
        drop(unique);
        // This is an EXPLICIT inspection-bank choice for the eight-heading
        // asset/math test, not a runtime fallback or a claim that the complete
        // shared scope has a unique file for all eight headings. Product
        // routing still needs an independently verified resource manifest.
        let mut archives = SharedSpriteArchives::open(
            &root,
            SpriteArchiveSelection::Explicit(SharedArchiveKind::Sprite),
        )
        .unwrap();
        for (key, heading) in [1, 3]
            .into_iter()
            .flat_map(|key| (0..8).map(move |heading| (key, heading)))
        {
            let loaded = preload_action(
                &source,
                &mut archives,
                ActorActionRequest {
                    sprite_id: 0,
                    key: ActionKey::new(key),
                    heading,
                },
                ActorAssetLimits::default(),
            )
            .unwrap();
            assert_eq!(loaded.provenance.graphics_id, 3225);
            assert_eq!(loaded.provenance.requested_sprite_id, 0);
            assert_eq!(loaded.provenance.definition_sprite_id, 0);
            assert_eq!(loaded.provenance.heading, heading);
            let animation = loaded.clock.animation();
            assert_eq!(animation.frames_per_second, 24);
            if key == 1 {
                assert_eq!(animation.label_bytes, b"attack");
                assert_eq!(
                    animation
                        .frames
                        .iter()
                        .map(|f| f.duration_ticks)
                        .collect::<Vec<_>>(),
                    vec![0, 3, 3, 3, 5, 2, 4]
                );
                assert_eq!(loaded.assets.sprites().len(), 2);
                assert_eq!(
                    loaded.assets.bindings()[0].asset.image_id,
                    u32::from(heading)
                );
                assert!(loaded.assets.bindings()[1..]
                    .iter()
                    .all(|b| b.asset.image_id == 16 + u32::from(heading)));
            } else {
                assert_eq!(animation.label_bytes, b"breath");
                assert_eq!(
                    animation
                        .frames
                        .iter()
                        .map(|f| f.duration_ticks)
                        .collect::<Vec<_>>(),
                    vec![4; 8]
                );
                assert_eq!(loaded.assets.sprites().len(), 1);
                assert!(loaded
                    .assets
                    .bindings()
                    .iter()
                    .all(|b| b.asset.image_id == 8 + u32::from(heading)));
            }
            for (index, binding) in loaded.assets.bindings().iter().enumerate() {
                assert_eq!(binding.asset.subframe, animation.frames[index].subframe);
                assert_eq!(binding.asset.graphics_id, 3225);
                assert!(!loaded.assets.frame(index).unwrap().rgba.is_empty());
            }
            if heading != 0 {
                continue;
            }
            eprintln!(
                "sprite0 action{key}: label={:?} fps={} ticks={:?}; assets={:?}; pixels={}",
                String::from_utf8_lossy(&loaded.clock.animation().label_bytes),
                loaded.clock.animation().frames_per_second,
                loaded
                    .clock
                    .animation()
                    .frames
                    .iter()
                    .map(|f| f.duration_ticks)
                    .collect::<Vec<_>>(),
                loaded
                    .assets
                    .sprites()
                    .iter()
                    .map(|s| (&s.archive, &s.name, s.decoded.frames.len()))
                    .collect::<Vec<_>>(),
                loaded.assets.decoded_pixels()
            );
            // Independently generated by existing Lin.Helper.Core.dll 1.5.4
            // SprReader.LoadRaw on these explicit Sprite entries. Compare
            // every decoded subframe's bounds and concatenated true RGBA,
            // not only successful parsing or a preview screenshot.
            for sprite in loaded.assets.sprites() {
                assert_eq!(sprite.archive, SharedArchiveKind::Sprite);
                let (input_hash, rgba_hash, bounds) = match sprite.name.as_str() {
                    "3225-0.spr" => (
                        "CBF1D6832B05DD5B86A14F5A4DDA9C783F42FEAE7A06963ACAD80F20A87F6EEC",
                        "23BAC8AD9536413BDC6D07F13AA60E749701B70B93F31BA57B4131D009EF9EC6",
                        vec![
                            (0, -48, 48, 72),
                            (0, -48, 48, 72),
                            (-24, -60, 48, 84),
                            (-24, -60, 48, 72),
                        ],
                    ),
                    "3225-16.spr" => (
                        "B3E3FAFA4CA03941E4B659F29D14D3E3899739650A996F8CB8C73154E1079AB2",
                        "296CEE3DE46CFE451CCA638ED6531C516EBDD5917292404D3723F73AD0A5663A",
                        vec![
                            (0, -48, 72, 84),
                            (0, -48, 72, 84),
                            (0, -36, 72, 72),
                            (0, -48, 72, 84),
                            (0, -48, 72, 84),
                            (0, -48, 72, 84),
                        ],
                    ),
                    "3225-8.spr" => (
                        "06BDB6C76AA963C412EE0210E5A00DB06FEE8AD2CD8BC3DCEC9D6ECA8F226078",
                        "3A1BC741D2654B6CE216D21A203ABB77EFC87BA3B19BC89D596AA28FACBB5EE5",
                        vec![(0, -48, 48, 72); 8],
                    ),
                    other => panic!("unexpected graphics-bank asset {other}"),
                };
                let payload = archives
                    .archives
                    .iter_mut()
                    .find(|a| a.kind() == sprite.archive)
                    .unwrap()
                    .read_entry(&sprite.name)
                    .unwrap();
                assert_eq!(format!("{:X}", Sha256::digest(&payload)), input_hash);
                let mut rgba = Sha256::new();
                for frame in &sprite.decoded.frames {
                    rgba.update(&frame.rgba);
                }
                assert_eq!(
                    format!("{:X}", rgba.finalize()),
                    rgba_hash,
                    "{} RGBA",
                    sprite.name
                );
                assert_eq!(
                    sprite
                        .decoded
                        .frames
                        .iter()
                        .map(|frame| (frame.x_offset, frame.y_offset, frame.width, frame.height))
                        .collect::<Vec<_>>(),
                    bounds
                );
            }
        }
    }
}
