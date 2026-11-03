//! Shared-resource world presentation, independent of native BIN/window/network.
//!
//! Logical base-character IDs are from the local native C_CreateChar.java
//! lines 104–105 and L1PcInstance CLASSID constants, also BattleClassGfx.java.
//! A logical ID selects shared Text/list.spr; its graphics alias and each
//! action's image/subframe select actual SPRs. Never substitute universal gfx3225.
//! `load`/`load_combat` preserve their unarmed contract. `load_combat_armed`
//! separately qualifies complete shared class/weapon families; unsupported
//! pairs never substitute unarmed art. Transformations, random extra attacks,
//! extra effects, full native blending and actor-update history remain
//! incomplete; map and sprite fragments share the qualified native cell queue.
//!
//! SPR routing has direct native evidence, not an icon analogy:
//! ida-client-oep-v2/full-disassembly.asm sub_4406C0 -> sub_440010 formats
//! %d-%d.spr, hashes ASCII sum modulo 16 via sub_43FFC0, reads SpriteNN then
//! Sprite on member absence. The archive bindings are at 0x42ECD3/0x42ED98.
//! Corruption, ambiguity, missing archives or decoding errors are not misses.
//!
//! `load` belongs on an owned cancellable resource worker. It retains no file
//! handles in the completed immutable assets. `render_into` does no I/O and is
//! intended for an owned render worker, not WM_PAINT. Cancellation checks bound
//! CPU stages; synchronous filesystem operations are not hard-deadline promises.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};

use super::{
    animation::{AnimationClock, PlaybackLimits},
    combat_animation::{qualify_weapon_appearance, CombatAction, QualifiedCombatAnimation},
    compositor::Canvas,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    scene::{static_draw_list, DrawTile},
    shared_action_catalog::SharedActionCatalog,
    sprite::{decode_world_spr_with_limits, DecodedSprite, SpriteLimits},
    sprite_actions::{ActionKey, Directionality, FrameEvent},
    tile::{decode_classic_til_selected, decode_roof_til_selected, TileLimits},
    world_map::{ArtLimitations, PixelPoint, ViewProjection, WorldCell, WorldMap},
};

pub const MAX_ACTORS: usize = 100;
const MAX_APPEARANCES: usize = MAX_ACTORS;
const MAX_DRAW_TILES: usize = 1_000_000;
const MAX_SOURCE_ENTRY_BYTES: usize = 8 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;
const MAX_SPRITE_ENTRIES: usize = 4096;
const MAX_INDEX_RECORDS: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BaseAppearance {
    pub class: u8,
    pub sex: u8,
}

impl BaseAppearance {
    pub fn logical_sprite_id(self) -> Result<u32> {
        const MALE: [u32; 7] = [0, 61, 138, 734, 2786, 6658, 6671];
        const FEMALE: [u32; 7] = [1, 48, 37, 1186, 2796, 6661, 6650];
        ensure!(
            self.sex <= 1 && self.class <= 6,
            "unsupported base character class/sex"
        );
        let ids = if self.sex == 0 { MALE } else { FEMALE };
        Ok(ids[self.class as usize])
    }
}

/// Asset-cache identity must include the weapon: two same-class players may
/// hold different weapons. Source actor IDs remain separate from this art key.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CombatAppearance {
    pub appearance: BaseAppearance,
    pub weapon_type: u8,
}
impl CombatAppearance {
    pub fn action_key(self, action: BaseAction) -> Result<ActionKey> {
        let base = qualify_weapon_appearance(self.appearance, self.weapon_type)?;
        Ok(ActionKey::new(match action {
            BaseAction::Walk => base,
            BaseAction::Attack => base + 1,
            BaseAction::Damage => base + 2,
            BaseAction::Idle => base + 3,
            BaseAction::Death => 8,
        }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BaseAction {
    Idle,
    Walk,
    Attack,
    Damage,
    Death,
}

impl BaseAction {
    pub fn action_key(self) -> ActionKey {
        ActionKey::new(match self {
            Self::Idle => 3,
            Self::Walk => 0,
            Self::Attack => 1,
            Self::Damage => 2,
            Self::Death => 8,
        })
    }
    pub fn combat(self) -> Option<CombatAction> {
        match self {
            Self::Attack => Some(CombatAction::Attack),
            Self::Damage => Some(CombatAction::Damage),
            Self::Death => Some(CombatAction::Death),
            _ => None,
        }
    }
}

/// Presentation receives confirmed world positions. It never predicts movement,
/// changes occupancy, starts an attack or writes authority state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActorState {
    pub kind: super::participant::MemberKind,
    pub entity_id: u64,
    pub source_id: u32,
    pub object_id: u64,
    pub appearance: BaseAppearance,
    /// Confirmed retained weapon type, not a native action byte or inventory ID.
    pub weapon_type: u8,
    pub cell: WorldCell,
    pub heading: u8,
    pub action: BaseAction,
    /// Time since this authoritative presentation action began; not packet age
    /// or a wall-clock timestamp. Frame phase uses the source FPS and ticks.
    pub action_elapsed: Duration,
}

/// Owned immutable input. The caller's per-match session controller remains
/// responsible for monotonic acceptance and suppressing late generations.
#[derive(Clone, Debug)]
pub struct SceneSnapshot {
    world_id: u64,
    revision: u64,
    map_digest: [u8; 32],
    selected_entity_id: u64,
    actors: Vec<ActorState>,
    pet: Option<PetScene>,
    hidden: BTreeSet<u64>,
}

/// Race identities are never interpreted as a player class/sex animation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PetRunner {
    pub entity_id: u64,
    pub appearance: super::pet_animation::PetAppearance,
    pub sequence: u64,
    pub step_interval_nanos: u64,
    pub competing: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PetScene {
    pub generation: u64,
    pub course: [u8; 32],
    pub runners: Vec<PetRunner>,
}

impl SceneSnapshot {
    pub fn new(
        world_id: u64,
        revision: u64,
        map_digest: [u8; 32],
        selected_entity_id: u64,
        actors: Vec<ActorState>,
    ) -> Result<Self> {
        Self::new_inner(
            world_id,
            revision,
            map_digest,
            selected_entity_id,
            actors,
            None,
            &[],
        )
    }
    pub(crate) fn new_pet(
        world_id: u64,
        revision: u64,
        map_digest: [u8; 32],
        selected_entity_id: u64,
        actors: Vec<ActorState>,
        pet: PetScene,
    ) -> Result<Self> {
        Self::new_inner(
            world_id,
            revision,
            map_digest,
            selected_entity_id,
            actors,
            Some(pet),
            &[],
        )
    }
    /// Presentation visibility, not participant removal or command authority.
    /// Retain the selected owner's data/camera even when that owner departed.
    pub(crate) fn new_with_hidden(
        world_id: u64,
        revision: u64,
        map_digest: [u8; 32],
        selected_entity_id: u64,
        actors: Vec<ActorState>,
        hidden: &[u64],
    ) -> Result<Self> {
        Self::new_inner(
            world_id,
            revision,
            map_digest,
            selected_entity_id,
            actors,
            None,
            hidden,
        )
    }
    pub(crate) fn with_actors(&self, actors: Vec<ActorState>) -> Result<Self> {
        Self::new_inner(
            self.world_id,
            self.revision,
            self.map_digest,
            self.selected_entity_id,
            actors,
            self.pet.clone(),
            &self.hidden.iter().copied().collect::<Vec<_>>(),
        )
    }
    pub(crate) fn hidden(&self, entity: u64) -> bool {
        self.hidden.contains(&entity)
    }
    fn new_inner(
        world_id: u64,
        revision: u64,
        map_digest: [u8; 32],
        selected_entity_id: u64,
        mut actors: Vec<ActorState>,
        mut pet: Option<PetScene>,
        hidden: &[u64],
    ) -> Result<Self> {
        ensure!(
            positive_id(world_id) && positive_id(revision),
            "invalid world identity/revision"
        );
        ensure!(positive_id(selected_entity_id), "invalid selected entity");
        ensure!(
            !actors.is_empty() && actors.len() <= MAX_ACTORS,
            "world actor count exceeds bounds"
        );
        let mut entities = BTreeSet::new();
        let mut characters = BTreeSet::new();
        let mut cells = BTreeSet::new();
        let hidden_set = hidden.iter().copied().collect::<BTreeSet<_>>();
        ensure!(
            hidden_set.len() == hidden.len()
                && hidden_set
                    .iter()
                    .all(|id| actors.iter().any(|a| a.entity_id == *id)),
            "unknown/duplicate hidden actor"
        );
        if let Some(pet) = &mut pet {
            ensure!(
                positive_id(pet.generation)
                    && pet.course != [0; 32]
                    && (2..=10).contains(&actors.len())
                    && pet.runners.len() == actors.len(),
                "invalid pet scene binding"
            );
            pet.runners.sort_by_key(|r| r.entity_id);
            let mut ids = BTreeSet::new();
            for runner in &pet.runners {
                ensure!(
                    ids.insert(runner.entity_id)
                        && runner.step_interval_nanos > 0
                        && actors.iter().any(|a| a.entity_id == runner.entity_id)
                        && runner.appearance == pet.runners[0].appearance,
                    "pet roster/form binding changed"
                );
            }
        }
        for actor in &actors {
            ensure!(
                positive_id(actor.entity_id)
                    && positive_id(actor.object_id)
                    && actor.source_id > 0
                    && actor.source_id <= i32::MAX as u32,
                "invalid actor identity"
            );
            if pet.is_none() {
                qualify_weapon_appearance(actor.appearance, actor.weapon_type)?;
            } else {
                ensure!(
                    actor.kind == super::participant::MemberKind::Human && actor.weapon_type == 0,
                    "pet scene cannot carry combat stances"
                );
            }
            ensure!(actor.heading < 8, "actor heading outside 0..7");
            ensure!(entities.insert(actor.entity_id), "duplicate world entity");
            ensure!(
                characters.insert((actor.kind, actor.source_id, actor.object_id)),
                "duplicate source character in world"
            );
            if !hidden_set.contains(&actor.entity_id)
                && pet.as_ref().is_none_or(|p| {
                    p.runners
                        .iter()
                        .any(|r| r.entity_id == actor.entity_id && r.competing)
                })
            {
                ensure!(cells.insert(actor.cell), "duplicate occupied world cell");
            }
        }
        ensure!(
            actors
                .iter()
                .any(|actor| actor.entity_id == selected_entity_id
                    && actor.kind == super::participant::MemberKind::Human),
            "selected actor absent from world"
        );
        actors.sort_by_key(|actor| actor.entity_id);
        Ok(Self {
            world_id,
            revision,
            map_digest,
            selected_entity_id,
            actors,
            pet,
            hidden: hidden_set,
        })
    }
    pub fn world_id(&self) -> u64 {
        self.world_id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn map_digest(&self) -> [u8; 32] {
        self.map_digest
    }
    pub fn selected_entity_id(&self) -> u64 {
        self.selected_entity_id
    }
    pub fn actors(&self) -> &[ActorState] {
        &self.actors
    }
    pub(crate) fn pet(&self) -> Option<&PetScene> {
        self.pet.as_ref()
    }
}

fn positive_id(value: u64) -> bool {
    value > 0 && value <= i64::MAX as u64
}
fn cancelled(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "world scene loading/rendering cancelled"
    );
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub struct SceneLimits {
    /// Cumulative encoded Tile/SPR bytes, including failed stages, before reads.
    pub max_encoded_bytes: usize,
    /// Retained Tile pixel storage/row masks + SPR assets, not per actor/action.
    pub max_texture_bytes: usize,
    pub max_draw_tiles: usize,
}

impl Default for SceneLimits {
    fn default() -> Self {
        Self {
            max_encoded_bytes: 32 * 1024 * 1024,
            max_texture_bytes: 128 * 1024 * 1024,
            max_draw_tiles: MAX_DRAW_TILES,
        }
    }
}

impl SceneLimits {
    fn validate(self) -> Result<Self> {
        ensure!(
            self.max_encoded_bytes > 0
                && self.max_encoded_bytes <= 256 * 1024 * 1024
                && self.max_texture_bytes >= 24 * 24 * 4
                && self.max_texture_bytes <= 256 * 1024 * 1024
                && self.max_draw_tiles > 0
                && self.max_draw_tiles <= MAX_DRAW_TILES,
            "invalid bounded world scene limits"
        );
        Ok(self)
    }
}

struct Budget {
    limits: SceneLimits,
    encoded: usize,
    decoded: usize,
}
impl Budget {
    fn encoded_entry(&mut self, bytes: u32) -> Result<()> {
        let bytes = bytes as usize;
        ensure!(
            bytes > 0 && bytes <= MAX_SOURCE_ENTRY_BYTES,
            "world asset exceeds single-entry byte limit"
        );
        let total = self
            .encoded
            .checked_add(bytes)
            .context("world encoded budget overflow")?;
        ensure!(
            total <= self.limits.max_encoded_bytes,
            "world encoded asset budget exceeded"
        );
        self.encoded = total;
        Ok(())
    }
    fn remaining_pixels(&self) -> Result<usize> {
        let pixels = (self.limits.max_texture_bytes - self.decoded) / 4;
        ensure!(pixels > 0, "world decoded texture budget exhausted");
        Ok(pixels)
    }
    fn decoded_entry(&mut self, bytes: usize) -> Result<()> {
        let total = self
            .decoded
            .checked_add(bytes)
            .context("world decoded budget overflow")?;
        ensure!(
            total <= self.limits.max_texture_bytes,
            "world decoded texture budget exceeded"
        );
        self.decoded = total;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteSource {
    pub archive: SharedArchiveKind,
    pub name: String,
    pub encoded_bytes: u32,
    pub sha256: [u8; 32],
}

struct SpriteAsset {
    source: SpriteSource,
    decoded: DecodedSprite,
}

/// Private loader-only handles. Only an exact member absence may choose base.
struct SpriteReader {
    root: PathBuf,
    base: SharedArchive,
    shards: BTreeMap<u8, SharedArchive>,
    records: usize,
}
impl SpriteReader {
    fn open(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        cancelled(cancel)?;
        let base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        cancelled(cancel)?;
        let records = base.len();
        ensure!(
            records <= MAX_INDEX_RECORDS,
            "world Sprite indexes exceed record budget"
        );
        Ok(Self {
            root: base.root().to_owned(),
            base,
            shards: BTreeMap::new(),
            records,
        })
    }
    fn read(
        &mut self,
        name: &str,
        budget: &mut Budget,
        cancel: &AtomicBool,
    ) -> Result<(SpriteSource, Vec<u8>)> {
        cancelled(cancel)?;
        let shard = sprite_shard(name);
        if !self.shards.contains_key(&shard) {
            let archive = SharedArchive::open(&self.root, SharedArchiveKind::SpriteShard(shard))?;
            self.records = self
                .records
                .checked_add(archive.len())
                .context("Sprite index overflow")?;
            ensure!(
                self.records <= MAX_INDEX_RECORDS,
                "world Sprite indexes exceed record budget"
            );
            self.shards.insert(shard, archive);
            cancelled(cancel)?;
        }
        let (kind, size) = route_sprite_member(
            shard,
            self.shards[&shard].entry(name).map(|entry| entry.size),
            || self.base.entry(name).map(|entry| entry.size),
        )
        .with_context(|| format!("locate shared SPR {name}"))?;
        let archive = if kind == SharedArchiveKind::Sprite {
            &mut self.base
        } else {
            self.shards.get_mut(&shard).unwrap()
        };
        budget.encoded_entry(size)?;
        cancelled(cancel)?;
        let bytes = archive.read_entry(name)?;
        cancelled(cancel)?;
        ensure!(
            bytes.len() == size as usize,
            "world SPR payload length changed"
        );
        Ok((
            SpriteSource {
                archive: archive.kind(),
                name: name.to_owned(),
                encoded_bytes: size,
                sha256: Sha256::digest(&bytes).into(),
            },
            bytes,
        ))
    }
}

fn sprite_shard(name: &str) -> u8 {
    // Input comes only from decimal graphics/image IDs and the ASCII suffix.
    (name.bytes().map(u32::from).sum::<u32>() & 15) as u8
}

fn route_sprite_member(
    shard: u8,
    candidate: std::result::Result<u32, ResourceError>,
    base: impl FnOnce() -> std::result::Result<u32, ResourceError>,
) -> std::result::Result<(SharedArchiveKind, u32), ResourceError> {
    match candidate {
        Ok(size) => Ok((SharedArchiveKind::SpriteShard(shard), size)),
        Err(ResourceError::MissingEntry(_)) => base().map(|size| (SharedArchiveKind::Sprite, size)),
        Err(error) => Err(error),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnimationInfo {
    pub appearance: BaseAppearance,
    pub logical_sprite_id: u32,
    pub graphics_id: u32,
    pub action: BaseAction,
    pub heading: u8,
    pub frame_count: usize,
    pub frames_per_second: u32,
    pub total_ticks: u64,
}

struct FrameBinding {
    sprite: String,
    subframe: usize,
}
struct AnimationAssets {
    info: AnimationInfo,
    ends: Vec<u64>,
    frames: Vec<FrameBinding>,
}
impl AnimationAssets {
    fn source_frame(&self, elapsed: Duration) -> usize {
        if self.info.action.combat().is_some() {
            let initial = self.ends[0];
            let active = self.info.total_ticks - initial;
            let tick = elapsed.as_nanos() * u128::from(self.info.frames_per_second) / 1_000_000_000;
            if tick >= u128::from(active) {
                return 0;
            }
            return 1 + self.ends[1..].partition_point(|end| *end - initial <= tick as u64);
        }
        // Duration max * FPS<=1000 fits u128; modulus bounds even huge focus gaps.
        let tick = (elapsed.as_nanos() * u128::from(self.info.frames_per_second) / 1_000_000_000
            % u128::from(self.info.total_ticks)) as u64;
        self.ends.partition_point(|end| *end <= tick)
    }
}

struct WorldDraw {
    tile: DrawTile,
    roof_group: Option<super::world_roofs::Group>,
    #[cfg_attr(not(test), allow(dead_code))]
    sector_origin: WorldCell,
    native: super::world_depth::NativeDrawCell,
}

impl WorldDraw {
    fn new(tile: DrawTile, sector_origin: WorldCell) -> Result<Self> {
        let origin = super::world_depth::NativeDrawCell::from_world(sector_origin)
            .context("sector native origin overflow")?;
        let native = super::world_depth::NativeDrawCell {
            half_x: origin
                .half_x
                .checked_add(i32::from(tile.half_x))
                .context("map half-X overflow")?,
            y: origin
                .y
                .checked_add(i32::from(tile.half_y))
                .context("map Y overflow")?,
        };
        Ok(Self {
            tile,
            roof_group: None,
            sector_origin,
            native,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneLimitations {
    pub opacity_records: usize,
    pub sprite_effects: usize,
    pub portals: usize,
    /// Production uses native per-fragment half-cell ordering. Synthetic
    /// flattened-only test fixtures still advertise the old limitation.
    pub actor_object_occlusion_unimplemented: bool,
    pub actor_dependent_roofs_unimplemented: bool,
    /// Kinds 0/1 use qualified linkage/weights. Exact native delayed timer
    /// scheduling and cross-group pointer alias side effects are not emulated.
    pub native_roof_timers_and_aliases_unverified: bool,
    pub native_actor_update_order_unverified: bool,
    pub weapon_and_transformation_art_unimplemented: bool,
    /// Static packet/draw evidence qualifies (-24,-12) from whole-cell center
    /// to even half-tile origin. Live native visual equivalence remains a
    /// separate uncompleted acceptance gate, not a guessed frame-size pivot.
    pub native_actor_foot_pivot_unverified: bool,
    /// Source walk cues are retained as IDs, not silently claimed to play.
    pub unplayed_sound_ids: Vec<u32>,
    /// Exact observed `<478`/`<479` cues on the fourteen unarmed Walk0
    /// definitions. Historical parser/API name only: the retained native BIN
    /// routes BOTH `<` and `[` through sound/%d.wav with different voice rules.
    /// These remain unplayed audio groups, not visual SPR effects. Do not guess
    /// SPR IDs or treat the two marker groups as identical playback semantics.
    pub unrendered_base_effect_ids: Vec<u32>,
    pub unrendered_base_effects: Vec<UnrenderedBaseEffect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnrenderedBaseEffect {
    pub appearance: BaseAppearance,
    pub action: BaseAction,
    pub frame_index: usize,
    pub tick: u64,
    pub effect_id: u32,
}

fn retain_base_cue(
    appearance: BaseAppearance,
    action: BaseAction,
    frame_index: usize,
    tick: u64,
    event: &FrameEvent,
    sounds: &mut BTreeSet<u32>,
    base_effects: &mut Vec<UnrenderedBaseEffect>,
) -> Result<()> {
    appearance.logical_sprite_id()?;
    match event {
        FrameEvent::Sound(id) => {
            sounds.insert(*id);
        }
        FrameEvent::Effect(id @ (478 | 479)) if action == BaseAction::Walk => {
            // Preserve the source marker/provenance. Native sound-group dispatch
            // is separate from this historical FrameEvent::Effect parser name.
            base_effects.push(UnrenderedBaseEffect {
                appearance,
                action,
                frame_index,
                tick,
                effect_id: *id,
            });
        }
        _ => anyhow::bail!(
            "unsupported base cue {appearance:?}/{action:?}/frame{frame_index}: {event:?}"
        ),
    }
    Ok(())
}

/// Immutable, owned asset set. All sectors of this explicit geometry manifest
/// are preloaded so a confirmed camera move never opens an archive on rendering.
pub struct WorldSceneAssets {
    map: Arc<WorldMap>,
    roofs: super::world_roofs::RoofMap,
    draws: Vec<WorldDraw>,
    tiles: BTreeMap<u16, BTreeMap<usize, super::world_raster::PreparedTile>>,
    sprites: BTreeMap<String, SpriteAsset>,
    animations: BTreeMap<(CombatAppearance, BaseAction, u8), AnimationAssets>,
    /// Retains every original source cue, including zero-tick audio groups and
    /// Hit. Rendering below never plays these cues or applies combat damage.
    combat: BTreeMap<(CombatAppearance, BaseAction), QualifiedCombatAnimation>,
    walks: BTreeMap<CombatAppearance, super::fixed_walk::QualifiedFixedWalk>,
    limitations: SceneLimitations,
    encoded_bytes: usize,
    texture_bytes: usize,
}

impl WorldSceneAssets {
    pub fn load(
        root: &Path,
        map: Arc<WorldMap>,
        appearances: &[BaseAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::load_unarmed(root, map, appearances, limits, cancel, false)
    }

    /// Base/unarmed combat art only. Does not substitute these actions for any
    /// equipped weapon stance; callers must explicitly gate weapon_type != 0.
    pub fn load_combat(
        root: &Path,
        map: Arc<WorldMap>,
        appearances: &[BaseAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::load_unarmed(root, map, appearances, limits, cancel, true)
    }

    fn load_unarmed(
        root: &Path,
        map: Arc<WorldMap>,
        appearances: &[BaseAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
        include_combat: bool,
    ) -> Result<Self> {
        ensure!(
            appearances.len() <= MAX_ACTORS,
            "world appearance input count exceeds bounds"
        );
        let armed: Vec<_> = appearances
            .iter()
            .map(|&appearance| CombatAppearance {
                appearance,
                weapon_type: 0,
            })
            .collect();
        Self::load_kind(root, map, &armed, limits, cancel, include_combat)
    }

    /// Preloads each exact class/sex/weapon family, all five actions and eight
    /// headings under one cumulative byte/pixel/entry budget. No game authority.
    pub fn load_combat_armed(
        root: &Path,
        map: Arc<WorldMap>,
        appearances: &[CombatAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::load_kind(root, map, appearances, limits, cancel, true)
    }

    /// Reuses one renderer-local pinned Text catalog. The completed assets do
    /// not retain the catalog, its AST, or its open archive handles.
    pub(crate) fn load_with_catalog(
        catalog: &SharedActionCatalog,
        map: Arc<WorldMap>,
        appearances: &[BaseAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        ensure!(
            appearances.len() <= MAX_ACTORS,
            "world appearance input count exceeds bounds"
        );
        let armed: Vec<_> = appearances
            .iter()
            .map(|&appearance| CombatAppearance {
                appearance,
                weapon_type: 0,
            })
            .collect();
        Self::load_kind_with_catalog(catalog, map, &armed, limits, cancel, false)
    }

    pub(crate) fn load_combat_armed_with_catalog(
        catalog: &SharedActionCatalog,
        map: Arc<WorldMap>,
        appearances: &[CombatAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::load_kind_with_catalog(catalog, map, appearances, limits, cancel, true)
    }

    fn validate_load(
        appearances: &[CombatAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
    ) -> Result<(SceneLimits, BTreeSet<CombatAppearance>)> {
        cancelled(cancel)?;
        let limits = limits.validate()?;
        ensure!(
            !appearances.is_empty() && appearances.len() <= MAX_ACTORS,
            "world appearance input count exceeds bounds"
        );
        for appearance in appearances {
            qualify_weapon_appearance(appearance.appearance, appearance.weapon_type)?;
        }
        let appearances: BTreeSet<_> = appearances.iter().copied().collect();
        ensure!(
            appearances.len() <= MAX_APPEARANCES,
            "unsupported appearance count"
        );
        Ok((limits, appearances))
    }

    fn load_kind(
        root: &Path,
        map: Arc<WorldMap>,
        appearances: &[CombatAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
        include_combat: bool,
    ) -> Result<Self> {
        // Preserve the standalone API's validation-before-I/O contract.
        let (limits, appearances) = Self::validate_load(appearances, limits, cancel)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        Self::load_validated(&catalog, map, appearances, limits, cancel, include_combat)
    }

    fn load_kind_with_catalog(
        catalog: &SharedActionCatalog,
        map: Arc<WorldMap>,
        appearances: &[CombatAppearance],
        limits: SceneLimits,
        cancel: &AtomicBool,
        include_combat: bool,
    ) -> Result<Self> {
        let (limits, appearances) = Self::validate_load(appearances, limits, cancel)?;
        Self::load_validated(catalog, map, appearances, limits, cancel, include_combat)
    }

    /// Terrain-only race path: no player appearance qualification or player SPR
    /// archive is opened. PetAssets owns its separate bounded body/shadow bank.
    pub(crate) fn load_terrain_with_catalog(
        shared: &SharedActionCatalog,
        map: Arc<WorldMap>,
        limits: SceneLimits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        cancelled(cancel)?;
        let limits = limits.validate()?;
        Self::load_validated(shared, map, BTreeSet::new(), limits, cancel, false)
    }

    fn load_validated(
        shared: &SharedActionCatalog,
        map: Arc<WorldMap>,
        appearances: BTreeSet<CombatAppearance>,
        limits: SceneLimits,
        cancel: &AtomicBool,
        include_combat: bool,
    ) -> Result<Self> {
        cancelled(cancel)?;
        ensure!(
            shared.byte_length() <= MAX_TEXT_BYTES,
            "shared list.spr exceeds world catalog budget"
        );
        let root = shared.root();
        let catalog = shared.catalog();
        let mut budget = Budget {
            limits,
            encoded: 0,
            decoded: 0,
        };
        let mut reader = if appearances.is_empty() {
            None
        } else {
            Some(SpriteReader::open(root, cancel)?)
        };
        let mut sprites = BTreeMap::new();
        let mut animations = BTreeMap::new();
        let mut combat = BTreeMap::new();
        let mut walks = BTreeMap::new();
        let mut sounds = BTreeSet::new();
        let mut base_effects = Vec::new();
        for armed in appearances {
            let appearance = armed.appearance;
            let logical = appearance.logical_sprite_id()?;
            walks.insert(
                armed,
                super::fixed_walk::QualifiedFixedWalk::from_player_catalog(
                    catalog,
                    appearance,
                    armed.weapon_type,
                )?,
            );
            let actions: &[BaseAction] = if include_combat {
                &[
                    BaseAction::Idle,
                    BaseAction::Walk,
                    BaseAction::Attack,
                    BaseAction::Damage,
                    BaseAction::Death,
                ]
            } else {
                &[BaseAction::Idle, BaseAction::Walk]
            };
            for &action in actions {
                if let Some(kind) = action.combat() {
                    combat.insert(
                        (armed, action),
                        QualifiedCombatAnimation::from_catalog_armed(
                            catalog,
                            appearance,
                            armed.weapon_type,
                            kind,
                        )?,
                    );
                }
                let resolved = catalog
                    .resolve_action(logical, armed.action_key(action)?)
                    .with_context(|| {
                        format!("unsupported shared base appearance {appearance:?}/{action:?}")
                    })?;
                let graphics = catalog.sprites[&logical].graphics_id;
                ensure!(
                    catalog.sprites[&resolved.definition_sprite_id].graphics_id == graphics,
                    "unverified cross-graphics action inheritance"
                );
                let clock = AnimationClock::with_limits(
                    resolved.animation,
                    PlaybackLimits {
                        max_frames: 256,
                        max_events: 1024,
                    },
                )?;
                let mut event_tick = 0;
                for (frame_index, frame) in clock.animation().frames.iter().enumerate() {
                    for event in &frame.events {
                        if action.combat().is_none() {
                            retain_base_cue(
                                appearance,
                                action,
                                frame_index,
                                event_tick,
                                event,
                                &mut sounds,
                                &mut base_effects,
                            )?;
                        }
                    }
                    event_tick += u64::from(frame.duration_ticks);
                }
                for heading in 0..8u8 {
                    cancelled(cancel)?;
                    let mut frames = Vec::with_capacity(clock.animation().frames.len());
                    let mut ends = Vec::with_capacity(clock.animation().frames.len());
                    let mut total = 0u64;
                    for frame in &clock.animation().frames {
                        let direction =
                            if clock.animation().directionality == Directionality::EightWay {
                                u32::from(heading)
                            } else {
                                0
                            };
                        let image = frame
                            .image_id
                            .checked_add(direction)
                            .context("actor image ID overflow")?;
                        let name = format!("{graphics}-{image}.spr");
                        if !sprites.contains_key(&name) {
                            ensure!(
                                sprites.len() < MAX_SPRITE_ENTRIES,
                                "world SPR cache entry budget exceeded"
                            );
                            let (source, bytes) = reader
                                .as_mut()
                                .context("player sprite reader absent")?
                                .read(&name, &mut budget, cancel)?;
                            let decoded = decode_world_spr_with_limits(
                                &bytes,
                                SpriteLimits {
                                    max_input_bytes: MAX_SOURCE_ENTRY_BYTES,
                                    max_total_pixels: budget.remaining_pixels()?,
                                    ..Default::default()
                                },
                            )
                            .with_context(|| {
                                format!("decode shared {:?}/{name}", source.archive)
                            })?;
                            cancelled(cancel)?;
                            let bytes = decoded
                                .retained_rgba_bytes()
                                .context("actor decoded byte overflow")?;
                            budget.decoded_entry(bytes)?;
                            sprites.insert(name.clone(), SpriteAsset { source, decoded });
                        }
                        ensure!(
                            (frame.subframe as usize) < sprites[&name].decoded.frames.len(),
                            "shared SPR {name} lacks requested subframe {} for {appearance:?}/{action:?}",
                            frame.subframe
                        );
                        frames.push(FrameBinding {
                            sprite: name,
                            subframe: frame.subframe as usize,
                        });
                        total = total
                            .checked_add(u64::from(frame.duration_ticks))
                            .context("frame tick overflow")?;
                        ends.push(total);
                    }
                    ensure!(
                        total == clock.total_ticks() && total > 0,
                        "invalid base animation duration"
                    );
                    animations.insert(
                        (armed, action, heading),
                        AnimationAssets {
                            info: AnimationInfo {
                                appearance,
                                logical_sprite_id: logical,
                                graphics_id: graphics,
                                action,
                                heading,
                                frame_count: frames.len(),
                                frames_per_second: clock.animation().frames_per_second,
                                total_ticks: total,
                            },
                            ends,
                            frames,
                        },
                    );
                }
            }
        }
        drop(reader);
        cancelled(cancel)?;
        let mut draws = Vec::new();
        let roofs = super::world_roofs::RoofMap::new(&map)?;
        let mut ids = BTreeMap::<u16, BTreeSet<usize>>::new();
        let mut raw_source_ids = BTreeMap::<u16, BTreeSet<usize>>::new();
        let mut limitations = SceneLimitations {
            opacity_records: 0,
            sprite_effects: 0,
            portals: 0,
            actor_object_occlusion_unimplemented: false,
            actor_dependent_roofs_unimplemented: false,
            native_roof_timers_and_aliases_unverified: true,
            native_actor_update_order_unverified: true,
            weapon_and_transformation_art_unimplemented: true,
            native_actor_foot_pivot_unverified: false,
            unplayed_sound_ids: sounds.into_iter().collect(),
            unrendered_base_effect_ids: base_effects
                .iter()
                .map(|event| event.effect_id)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            unrendered_base_effects: base_effects,
        };
        for binding in map.bindings() {
            cancelled(cancel)?;
            let sector = map
                .sector(binding.key)
                .context("bound world sector absent")?;
            let ArtLimitations {
                opacity_records,
                sprite_effects,
                portals,
                ..
            } = map.art_limitations(binding.key).unwrap();
            limitations.opacity_records += opacity_records;
            limitations.sprite_effects += sprite_effects;
            limitations.portals += portals;
            let count = sector
                .floor
                .len()
                .checked_add(sector.overlays.len())
                .and_then(|count| count.checked_add(sector.object_tile_count()))
                .context("world draw count overflow")?;
            ensure!(
                count <= limits.max_draw_tiles - draws.len(),
                "world draw-list budget exceeded"
            );
            let local = static_draw_list(sector)?;
            draws.try_reserve_exact(local.len())?;
            let (x, y) = binding.key.world_origin();
            let group_keys = std::iter::repeat_n(None, sector.floor.len() + sector.overlays.len())
                .chain(
                    sector
                        .objects
                        .iter()
                        .enumerate()
                        .flat_map(|(index, group)| {
                            std::iter::repeat_n(
                                Some(super::world_roofs::Group(
                                    binding.key.sector_x,
                                    binding.key.sector_y,
                                    index as u16,
                                )),
                                group.tiles.len(),
                            )
                        }),
                );
            for (tile, group) in local.into_iter().zip(group_keys) {
                if tile.layer == -2 || group.is_some_and(|group| roofs.controls(group)) {
                    raw_source_ids
                        .entry(tile.archive_entry_id)
                        .or_default()
                        .insert(tile.sub_index as usize);
                }
                ids.entry(tile.archive_entry_id)
                    .or_default()
                    .insert(tile.sub_index as usize);
                let mut draw = WorldDraw::new(tile, WorldCell { x, y })?;
                draw.roof_group = group;
                draws.push(draw);
            }
        }
        // Stable row index retains source insertion order within every cell.
        // Offscreen rows must not consume a full-scene scan on each 20Hz tick.
        draws.sort_by_key(|d| d.native.scan_row());
        let mut archive = SharedArchive::open(root, SharedArchiveKind::Tile)?;
        let mut tiles = BTreeMap::new();
        for (id, references) in ids {
            cancelled(cancel)?;
            let name = format!("{id}.til");
            let bytes = if map.bindings().len() > 16 {
                let input = super::tile_reader::load(
                    &mut archive,
                    &name,
                    &references,
                    budget.limits.max_encoded_bytes - budget.encoded,
                    cancel,
                )?;
                budget.encoded_entry(u32::try_from(input.read_bytes)?)?;
                input.data
            } else {
                // Preserve the qualified <=16-sector path and its accounting.
                budget.encoded_entry(archive.entry(&name)?.size)?;
                archive.read_entry(&name)?
            };
            cancelled(cancel)?;
            if map.bindings().len() > 16 {
                // Fixed staging: at most 64 normal + 64 roof 24x24 RGBA tiles
                // (288 KiB pixels), separate from the retained asset budget.
                // Every reference is decoded, with no viewport/map cropping.
                let indices: Vec<_> = references.iter().copied().collect();
                let mut prepared = BTreeMap::new();
                let input = super::tile::ValidatedTileInput::new(
                    &bytes,
                    TileLimits {
                        max_input_bytes: MAX_SOURCE_ENTRY_BYTES,
                        max_total_pixels: 64 * 24 * 24,
                        ..Default::default()
                    },
                )?;
                for batch in indices.chunks(64) {
                    cancelled(cancel)?;
                    let selected: BTreeSet<_> = batch.iter().copied().collect();
                    let decoded = input
                        .decode_selected(&selected, false)
                        .with_context(|| format!("decode shared Tile/{name}"))?;
                    let raw_ids: BTreeSet<_> = raw_source_ids
                        .get(&id)
                        .into_iter()
                        .flatten()
                        .copied()
                        .filter(|sub| {
                            selected.contains(sub) && decoded[sub].tile_type as u8 & 0x30 != 0
                        })
                        .collect();
                    let mut raw = if raw_ids.is_empty() {
                        BTreeMap::new()
                    } else {
                        input
                            .decode_selected(&raw_ids, true)
                            .with_context(|| format!("decode roof source Tile/{name}"))?
                    };
                    for (sub, tile) in decoded {
                        cancelled(cancel)?;
                        let tile = super::world_raster::PreparedTile::new(tile)?;
                        let tile = if let Some(raw) = raw.remove(&sub) {
                            tile.with_roof_raw(raw)?
                        } else {
                            tile
                        };
                        let tile = tile.compact_storage()?;
                        budget.decoded_entry(tile.retained_bytes())?;
                        prepared.insert(sub, tile);
                    }
                }
                tiles.insert(id, prepared);
                continue;
            }
            let decoded = decode_classic_til_selected(
                &bytes,
                TileLimits {
                    max_input_bytes: MAX_SOURCE_ENTRY_BYTES,
                    max_total_pixels: budget.remaining_pixels()?,
                    ..Default::default()
                },
                &references,
            )
            .with_context(|| format!("decode shared Tile/{name}"))?;
            cancelled(cancel)?;
            budget.decoded_entry(
                decoded
                    .values()
                    .try_fold(0usize, |sum, tile| {
                        sum.checked_add(tile.rgba.len() + super::compositor::FragmentRows::BYTES)
                    })
                    .context("tile cache byte overflow")?,
            )?;
            // Effect alpha/tint loses source RGB and even covered color keys.
            // Floors ignore overlay alpha; controlled roofs also need raw color.
            let raw_ids: BTreeSet<_> = raw_source_ids
                .get(&id)
                .into_iter()
                .flatten()
                .copied()
                .filter(|sub| decoded[sub].tile_type as u8 & 0x30 != 0)
                .collect();
            let mut roof_raw = if raw_ids.is_empty() {
                BTreeMap::new()
            } else {
                let raw = decode_roof_til_selected(
                    &bytes,
                    TileLimits {
                        max_input_bytes: MAX_SOURCE_ENTRY_BYTES,
                        max_total_pixels: budget.remaining_pixels()?,
                        ..Default::default()
                    },
                    &raw_ids,
                )
                .with_context(|| format!("decode roof source Tile/{name}"))?;
                budget.decoded_entry(
                    raw.len()
                        .checked_mul(24 * 24 * 4 + super::compositor::FragmentRows::BYTES)
                        .context("roof tile cache byte overflow")?,
                )?;
                raw
            };
            let prepared = decoded
                .into_iter()
                .map(|(sub, tile)| {
                    let prepared = super::world_raster::PreparedTile::new(tile)?;
                    Ok((
                        sub,
                        if let Some(raw) = roof_raw.remove(&sub) {
                            prepared.with_roof_raw(raw)?
                        } else {
                            prepared
                        },
                    ))
                })
                .collect::<Result<BTreeMap<_, _>>>()?;
            tiles.insert(id, prepared);
        }
        for draw in &draws {
            ensure!(
                tiles[&draw.tile.archive_entry_id].contains_key(&(draw.tile.sub_index as usize)),
                "shared Tile/{}.til lacks sub-index {}",
                draw.tile.archive_entry_id,
                draw.tile.sub_index
            );
        }
        drop(archive);
        cancelled(cancel)?;
        Ok(Self {
            map,
            roofs,
            draws,
            tiles,
            sprites,
            animations,
            combat,
            walks,
            limitations,
            encoded_bytes: budget.encoded,
            texture_bytes: budget.decoded,
        })
    }

    pub fn map(&self) -> &WorldMap {
        &self.map
    }
    pub(crate) fn sample_roofs(
        &self,
        animation: &mut super::world_roofs::Animation,
        snapshot: &SceneSnapshot,
        now: std::time::Instant,
    ) -> Result<super::world_roofs::Frame> {
        ensure!(
            snapshot.map_digest == self.map.canonical_digest(),
            "roof geometry digest mismatch"
        );
        let cell = snapshot
            .actors
            .iter()
            .find(|a| a.entity_id == snapshot.selected_entity_id)
            .context("roof selected actor absent")?
            .cell;
        animation.sample(
            &self.roofs,
            snapshot.world_id,
            snapshot.selected_entity_id,
            cell,
            now,
        )
    }
    pub fn combat_animation(
        &self,
        appearance: BaseAppearance,
        action: BaseAction,
    ) -> Option<&QualifiedCombatAnimation> {
        self.combat_animation_armed(appearance, 0, action)
    }
    pub fn combat_animation_armed(
        &self,
        appearance: BaseAppearance,
        weapon_type: u8,
        action: BaseAction,
    ) -> Option<&QualifiedCombatAnimation> {
        self.combat.get(&(
            CombatAppearance {
                appearance,
                weapon_type,
            },
            action,
        ))
    }
    pub fn combat_animations(
        &self,
    ) -> impl Iterator<Item = (CombatAppearance, BaseAction, &QualifiedCombatAnimation)> {
        self.combat
            .iter()
            .map(|(&(appearance, action), animation)| (appearance, action, animation))
    }
    pub(crate) fn player_walk_animation(
        &self,
        appearance: BaseAppearance,
        weapon_type: u8,
    ) -> Option<&super::fixed_walk::QualifiedFixedWalk> {
        self.walks.get(&CombatAppearance {
            appearance,
            weapon_type,
        })
    }
    pub(crate) fn player_walk_animations(
        &self,
    ) -> impl Iterator<Item = &super::fixed_walk::QualifiedFixedWalk> {
        self.walks.values()
    }
    pub fn limitations(&self) -> &SceneLimitations {
        &self.limitations
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }
    pub fn texture_bytes(&self) -> usize {
        self.texture_bytes
    }
    #[cfg(test)]
    pub(crate) fn decoded_tile_for_test(
        &self,
        id: u16,
        sub: usize,
    ) -> (super::tile::DecodedTile, Option<Vec<u8>>) {
        let tile = &self.tiles[&id][&sub];
        (
            tile.decoded_for_test(),
            tile.roof_raw_rgba().map(|b| b.into_owned()),
        )
    }
    pub fn tile_set_count(&self) -> usize {
        self.tiles.len()
    }
    pub fn sprite_sources(&self) -> impl Iterator<Item = &SpriteSource> {
        self.sprites.values().map(|sprite| &sprite.source)
    }
    pub fn animation_info(
        &self,
        appearance: BaseAppearance,
        action: BaseAction,
        heading: u8,
    ) -> Option<&AnimationInfo> {
        self.animation_info_armed(appearance, 0, action, heading)
    }
    pub fn animation_info_armed(
        &self,
        appearance: BaseAppearance,
        weapon_type: u8,
        action: BaseAction,
        heading: u8,
    ) -> Option<&AnimationInfo> {
        self.animations
            .get(&(
                CombatAppearance {
                    appearance,
                    weapon_type,
                },
                action,
                heading,
            ))
            .map(|animation| &animation.info)
    }

    /// Caller owns/reuses the surface. On failure/cancellation its partial
    /// pixels must not be published; swap only after this returns successfully.
    pub fn render_into(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
    ) -> Result<RenderInfo> {
        self.render_into_overrides(snapshot, canvas, cancel, &[])
    }

    /// A complete, snapshot-bound player appearance set. Polymorph layers use
    /// the actor's existing identity, depth owner and hit mask, never an NPC.
    pub(crate) fn render_player_appearances(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        forms: &super::player_appearance::Draws<'_>,
        walks: &[PlayerWalkPlacement],
    ) -> Result<RenderInfo> {
        self.render_player_appearances_effects(snapshot, canvas, cancel, forms, walks, &[], &[])
    }
    pub(crate) fn render_player_appearances_effects(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        forms: &super::player_appearance::Draws<'_>,
        walks: &[PlayerWalkPlacement],
        overrides: &[ActorOverride<'_>],
        effects: &[super::disintegrate_effect::DisintegrateDraw<'_>],
    ) -> Result<RenderInfo> {
        ensure!(
            snapshot.pet.is_none(),
            "player forms cannot replace pet resources"
        );
        self.render_layers(
            snapshot,
            canvas,
            cancel,
            overrides,
            &[],
            &[],
            None,
            &[],
            None,
            effects,
            walks,
            Some(forms),
        )
    }

    /// Qualified one-shot body replacement. There is still exactly one body
    /// and one truthful hit rectangle per actor, not a second painted sprite.
    pub fn render_into_overrides(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        overrides: &[ActorOverride<'_>],
    ) -> Result<RenderInfo> {
        self.render_into_objectives(snapshot, canvas, cancel, overrides, &[])
    }
    /// Objectives share the exact actor projection/depth sort, but are not
    /// participants and never enter appearance, character-key or roster checks.
    pub fn render_into_objectives(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        overrides: &[ActorOverride<'_>],
        objectives: &[ObjectiveDraw<'_>],
    ) -> Result<RenderInfo> {
        ensure!(
            snapshot.pet.is_none(),
            "pet scene requires the pet resource path"
        );
        self.render_layers(
            snapshot,
            canvas,
            cancel,
            overrides,
            objectives,
            &[],
            None,
            &[],
            None,
            &[],
            &[],
            None,
        )
    }
    #[cfg(test)]
    pub(crate) fn render_into_npcs(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        overrides: &[ActorOverride<'_>],
        objectives: &[ObjectiveDraw<'_>],
        npcs: &[NpcDraw<'_>],
    ) -> Result<RenderInfo> {
        self.render_into_npcs_roofs(snapshot, canvas, cancel, overrides, objectives, npcs, None)
    }
    #[cfg(test)]
    pub(crate) fn render_into_npcs_roofs(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        overrides: &[ActorOverride<'_>],
        objectives: &[ObjectiveDraw<'_>],
        npcs: &[NpcDraw<'_>],
        roofs: Option<&super::world_roofs::Frame>,
    ) -> Result<RenderInfo> {
        self.render_into_npcs_roofs_effects(
            snapshot,
            canvas,
            cancel,
            overrides,
            objectives,
            npcs,
            roofs,
            &[],
        )
    }
    /// Prepared effect layers enter the same half-cell queue as map/actors.
    /// No new server skill is enabled by this renderer entry point.
    pub(crate) fn render_into_npcs_roofs_effects(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        overrides: &[ActorOverride<'_>],
        objectives: &[ObjectiveDraw<'_>],
        npcs: &[NpcDraw<'_>],
        roofs: Option<&super::world_roofs::Frame>,
        effects: &[super::disintegrate_effect::DisintegrateDraw<'_>],
    ) -> Result<RenderInfo> {
        ensure!(
            snapshot.pet.is_none(),
            "NPC scene cannot use pet race resources"
        );
        self.render_layers(
            snapshot,
            canvas,
            cancel,
            overrides,
            objectives,
            &[],
            None,
            npcs,
            roofs,
            effects,
            &[],
            None,
        )
    }
    pub(crate) fn render_into_npcs_roofs_effects_walking(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        overrides: &[ActorOverride<'_>],
        objectives: &[ObjectiveDraw<'_>],
        npcs: &[NpcDraw<'_>],
        roofs: Option<&super::world_roofs::Frame>,
        effects: &[super::disintegrate_effect::DisintegrateDraw<'_>],
        walks: &[PlayerWalkPlacement],
    ) -> Result<RenderInfo> {
        ensure!(
            snapshot.pet.is_none(),
            "player walk cannot use pet resources"
        );
        self.render_layers(
            snapshot,
            canvas,
            cancel,
            overrides,
            objectives,
            &[],
            None,
            npcs,
            roofs,
            effects,
            walks,
            None,
        )
    }

    pub(crate) fn player_walk(
        &self,
        snapshot: &SceneSnapshot,
        entity: u64,
        from: WorldCell,
        elapsed: Duration,
        step_nanos: u64,
        cancel: &AtomicBool,
    ) -> Result<PlayerWalkPlacement> {
        cancelled(cancel)?;
        let actor = snapshot
            .actors
            .iter()
            .find(|a| a.entity_id == entity)
            .context("player walk actor absent")?;
        let form = CombatAppearance {
            appearance: actor.appearance,
            weapon_type: actor.weapon_type,
        };
        let walk = self.walks.get(&form).context("player walk not qualified")?;
        self.player_walk_qualified(snapshot, entity, from, elapsed, step_nanos, cancel, walk)
    }

    /// A logical-form-specific resource certificate. The renderer verifies the
    /// minted logical identity against either the base body or polymorph draw.
    pub(crate) fn player_walk_qualified(
        &self,
        snapshot: &SceneSnapshot,
        entity: u64,
        from: WorldCell,
        elapsed: Duration,
        step_nanos: u64,
        cancel: &AtomicBool,
        walk: &super::fixed_walk::QualifiedFixedWalk,
    ) -> Result<PlayerWalkPlacement> {
        cancelled(cancel)?;
        let actor = snapshot
            .actors
            .iter()
            .find(|a| a.entity_id == entity)
            .context("player walk actor absent")?;
        let form = CombatAppearance {
            appearance: actor.appearance,
            weapon_type: actor.weapon_type,
        };
        ensure!(
            snapshot.pet.is_none()
                && actor.action == BaseAction::Walk
                && step_nanos > 0
                && step_nanos <= 10_000_000_000
                && self.map.can_step(from, actor.cell)
                && super::world_map::DIRECTIONS[usize::from(actor.heading)]
                    == (actor.cell.x - from.x, actor.cell.y - from.y),
            "invalid player walk span"
        );
        // Same explicit modern pacing as the body clock: fit one native cycle
        // to one authority step. Native camera pulses move only the camera;
        // the shared SPR already contains the body displacement.
        let phase = elapsed
            .as_nanos()
            .min(u128::from(step_nanos))
            .checked_mul(walk.step_duration().as_nanos())
            .context("player walk phase overflow")?
            / u128::from(step_nanos);
        let sample = walk
            .start_unpaced_step(actor.heading)?
            .advance(Duration::from_nanos(u64::try_from(phase)?), cancel)?;
        let offset = sample.local_camera_pulses.iter().fold((0, 0), |(x, y), p| {
            (x + p.displacement.0, y + p.displacement.1)
        });
        Ok(PlayerWalkPlacement {
            world: snapshot.world_id,
            revision: snapshot.revision,
            entity,
            form,
            logical: walk.provenance().requested_sprite_id,
            heading: actor.heading,
            from,
            to: actor.cell,
            source: sample.source_frame,
            terminal: sample.terminal,
            offset,
        })
    }
    #[cfg(test)]
    pub(crate) fn render_into_pets(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        pets: &[PetDraw<'_>],
        camera: Option<ViewProjection>,
    ) -> Result<RenderInfo> {
        self.render_into_pets_roofs(snapshot, canvas, cancel, pets, camera, None)
    }
    pub(crate) fn render_into_pets_roofs(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        pets: &[PetDraw<'_>],
        camera: Option<ViewProjection>,
        roofs: Option<&super::world_roofs::Frame>,
    ) -> Result<RenderInfo> {
        let race = snapshot
            .pet
            .as_ref()
            .context("pet renderer lacks race binding")?;
        ensure!(
            self.animations.is_empty() && self.combat.is_empty() && self.sprites.is_empty(),
            "pet terrain must not retain player assets"
        );
        let mut ids = BTreeSet::new();
        ensure!(
            pets.len() == race.runners.iter().filter(|r| r.competing).count(),
            "pet draw roster differs from active authority"
        );
        for draw in pets {
            ensure!(
                ids.insert(draw.entity_id)
                    && race.runners.iter().any(|r| r.entity_id == draw.entity_id
                        && r.competing
                        && r.appearance.graphics_id() == draw.sample.body.graphics_id),
                "unknown/inactive pet draw or wrong form"
            );
        }
        self.render_layers(
            snapshot,
            canvas,
            cancel,
            &[],
            &[],
            pets,
            camera,
            &[],
            roofs,
            &[],
            &[],
            None,
        )
    }
    fn render_layers(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        overrides: &[ActorOverride<'_>],
        objectives: &[ObjectiveDraw<'_>],
        pets: &[PetDraw<'_>],
        camera: Option<ViewProjection>,
        npcs: &[NpcDraw<'_>],
        roofs: Option<&super::world_roofs::Frame>,
        effects: &[super::disintegrate_effect::DisintegrateDraw<'_>],
        walks: &[PlayerWalkPlacement],
        player_forms: Option<&super::player_appearance::Draws<'_>>,
    ) -> Result<RenderInfo> {
        cancelled(cancel)?;
        ensure!(
            player_forms.is_none_or(|forms| forms.matches(snapshot)),
            "stale player appearance frames"
        );
        ensure!(effects.len() <= 64, "disintegrate render event budget");
        let mut effect_ids = BTreeSet::new();
        for effect in effects {
            let binding = effect.binding();
            ensure!(
                binding.world_id == snapshot.world_id
                    && binding.map_digest == snapshot.map_digest
                    && binding.start_revision <= snapshot.revision
                    && effect_ids.insert(binding.event_id)
                    && self.map.standing(binding.target),
                "stale/duplicate/outside disintegrate draw"
            );
        }
        ensure!(
            objectives.len() <= 3, // Siege has exactly three separately typed towers.
            "objective presentation exceeds bound"
        );
        ensure!(
            overrides.len() <= snapshot.actors.len(),
            "too many actor replacements"
        );
        let mut replacement_ids = BTreeSet::new();
        for replacement in overrides {
            ensure!(
                replacement_ids.insert(replacement.entity_id)
                    && player_forms
                        .is_none_or(|forms| forms.frame(replacement.entity_id).is_none())
                    && snapshot
                        .actors
                        .iter()
                        .any(|a| a.entity_id == replacement.entity_id),
                "duplicate/unknown actor replacement"
            );
        }
        ensure!(
            canvas.width() <= 4096 && canvas.height() <= 4096,
            "world viewport exceeds limits"
        );
        ensure!(
            snapshot.map_digest == self.map.canonical_digest(),
            "world snapshot geometry digest mismatch"
        );
        let selected = snapshot
            .actors
            .iter()
            .find(|actor| actor.entity_id == snapshot.selected_entity_id)
            .context("selected actor missing")?;
        ensure!(
            walks.len() <= snapshot.actors.len() && (walks.is_empty() || snapshot.pet.is_none()),
            "invalid player walk roster"
        );
        let mut walk_ids = BTreeSet::new();
        for walk in walks {
            ensure!(
                walk.world == snapshot.world_id
                    && walk.revision == snapshot.revision
                    && walk_ids.insert(walk.entity)
                    && !overrides.iter().any(|v| v.entity_id == walk.entity)
                    && snapshot.actors.iter().any(|a| a.entity_id == walk.entity
                        && a.cell == walk.to
                        && a.heading == walk.heading
                        && a.action == BaseAction::Walk
                        && a.appearance == walk.form.appearance
                        && a.weapon_type == walk.form.weapon_type
                        && match player_forms.and_then(|forms| forms.frame(walk.entity)) {
                            Some(frame) =>
                                frame.form.logical_sprite_id() == walk.logical
                                    && frame.action == Some(BaseAction::Walk)
                                    && frame.source == walk.source,
                            None => a.appearance.logical_sprite_id().ok() == Some(walk.logical),
                        }),
                "stale or conflicting player walk placement"
            );
        }
        let camera = if let Some(walk) = walks
            .iter()
            .find(|w| w.entity == snapshot.selected_entity_id && !w.terminal)
        {
            ensure!(camera.is_none(), "conflicting player walk camera");
            Some(ViewProjection {
                anchor_cell: walk.from,
                anchor_pixel: PixelPoint {
                    x: (canvas.width() / 2) as i32 - walk.offset.0,
                    y: (canvas.height() / 2) as i32 - walk.offset.1,
                },
            })
        } else {
            camera
        };
        let projection = camera.unwrap_or(ViewProjection {
            anchor_cell: selected.cell,
            anchor_pixel: PixelPoint {
                x: (canvas.width() / 2) as i32,
                y: (canvas.height() / 2) as i32,
            },
        });
        let settled;
        let roofs = if let Some(roofs) = roofs {
            roofs
        } else {
            settled = self.roofs.settled(selected.cell)?;
            &settled
        };
        // Prove all required appearances/positions BEFORE touching the surface.
        let mut actor_plans = Vec::with_capacity(snapshot.actors.len() + objectives.len());
        for actor in &snapshot.actors {
            ensure!(
                self.map.standing(actor.cell),
                "authoritative actor position outside standable bound geometry"
            );
            if snapshot.hidden(actor.entity_id) {
                continue;
            }
            if snapshot.pet.is_some() {
                let Some(pet) = pets.iter().find(|p| p.entity_id == actor.entity_id) else {
                    continue;
                };
                ensure!(
                    self.map.standing(pet.anchor_cell),
                    "pet visual anchor outside geometry"
                );
                let frame = pet.sample.body.frame;
                let anchor = projection
                    .cell_to_pixel(pet.anchor_cell)
                    .context("pet projection overflow")?;
                let PixelPoint { x, y } = layer_position(anchor, frame)?;
                actor_plans.push((
                    anchor.y,
                    anchor.x,
                    actor.entity_id,
                    false,
                    frame,
                    Some(pet.sample.shadow.frame),
                    None,
                    ActorPlacement {
                        visibility: None,
                        entity_id: actor.entity_id,
                        cell: actor.cell,
                        anchor,
                        top_left: PixelPoint { x, y },
                        width: frame.width,
                        height: frame.height,
                        graphics_id: pet.sample.body.graphics_id,
                        source_frame: pet.sample.body.source_frame,
                        sprite_name: pet.sample.body.provenance.name.clone(),
                        subframe: pet.sample.body.subframe,
                    },
                ));
                continue;
            }
            if let Some(sample) = player_forms.and_then(|forms| forms.frame(actor.entity_id)) {
                let frame = sample.body.frame;
                let walk = walks.iter().find(|w| w.entity == actor.entity_id);
                let anchor = projection
                    .cell_to_pixel(walk.filter(|w| !w.terminal).map_or(actor.cell, |w| w.from))
                    .context("polymorph actor projection overflow")?;
                let top_left = layer_position(anchor, frame)?;
                actor_plans.push((
                    anchor.y,
                    anchor.x,
                    actor.entity_id,
                    false,
                    frame,
                    Some(sample.shadow.frame),
                    Some(sample.attachment.frame),
                    ActorPlacement {
                        visibility: None,
                        entity_id: actor.entity_id,
                        cell: actor.cell,
                        anchor,
                        top_left,
                        width: frame.width,
                        height: frame.height,
                        graphics_id: sample.body.asset.graphics_id,
                        source_frame: sample.source,
                        sprite_name: sample.body.provenance.name.clone(),
                        subframe: sample.body.asset.subframe as usize,
                    },
                ));
                continue;
            }
            let animation = self
                .animations
                .get(&(
                    CombatAppearance {
                        appearance: actor.appearance,
                        weapon_type: actor.weapon_type,
                    },
                    actor.action,
                    actor.heading,
                ))
                .with_context(|| {
                    format!(
                        "appearance/action/heading not preloaded: {:?}/{:?}/{}",
                        actor.appearance, actor.action, actor.heading
                    )
                })?;
            let walk = walks.iter().find(|w| w.entity == actor.entity_id);
            let index = walk.map_or_else(
                || animation.source_frame(actor.action_elapsed),
                |w| w.source,
            );
            let binding = &animation.frames[index];
            let sprite = &self.sprites[&binding.sprite];
            let replacement = overrides
                .iter()
                .find(|value| value.entity_id == actor.entity_id);
            let frame = replacement.map_or(&sprite.decoded.frames[binding.subframe], |value| {
                value.frame
            });
            let anchor = projection
                .cell_to_pixel(walk.filter(|w| !w.terminal).map_or(actor.cell, |w| w.from))
                .context("actor screen coordinate overflow")?;
            let PixelPoint { x, y } = layer_position(anchor, frame)?;
            actor_plans.push((
                anchor.y,
                anchor.x,
                actor.entity_id,
                false,
                frame,
                None,
                None,
                ActorPlacement {
                    visibility: None,
                    entity_id: actor.entity_id,
                    cell: actor.cell,
                    anchor,
                    top_left: PixelPoint { x, y },
                    width: frame.width,
                    height: frame.height,
                    graphics_id: replacement.map_or(animation.info.graphics_id, |r| r.graphics_id),
                    source_frame: replacement.map_or(index, |r| r.source_frame),
                    sprite_name: replacement
                        .map_or_else(|| sprite.source.name.clone(), |r| r.sprite_name.to_owned()),
                    subframe: replacement.map_or(binding.subframe, |r| r.subframe),
                },
            ));
        }
        let mut objective_ids = BTreeSet::new();
        for object in objectives {
            ensure!(
                positive_id(object.entity_id)
                    && objective_ids.insert(object.entity_id)
                    && !snapshot
                        .actors
                        .iter()
                        .any(|a| a.entity_id == object.entity_id
                            || object.blocking && a.cell == object.cell)
                    && self.map.standing(object.cell),
                "invalid objective identity or occupancy"
            );
            let anchor = projection
                .cell_to_pixel(object.cell)
                .context("objective screen coordinate overflow")?;
            let frame = object.frame;
            let PixelPoint { x, y } = layer_position(anchor, frame)?;
            actor_plans.push((
                anchor.y,
                anchor.x,
                object.entity_id,
                true,
                frame,
                object.shadow,
                None,
                ActorPlacement {
                    visibility: None,
                    entity_id: object.entity_id,
                    cell: object.cell,
                    anchor,
                    top_left: PixelPoint { x, y },
                    width: frame.width,
                    height: frame.height,
                    graphics_id: object.graphics_id,
                    source_frame: object.source_frame,
                    sprite_name: object.sprite_name.to_owned(),
                    subframe: object.subframe,
                },
            ));
        }
        let mut plans: Vec<_> = actor_plans
            .into_iter()
            .map(
                |(y, x, id, objective, frame, shadow, attachment, placement)| PaintPlan {
                    key: (y, x, id, 0),
                    frame,
                    shadow,
                    attachment,
                    placement: if objective {
                        Placement::Objective(placement)
                    } else {
                        Placement::Actor(placement)
                    },
                },
            )
            .collect();
        ensure!(npcs.len() <= 128, "NPC draw count exceeds bound");
        let mut npc_ids = std::collections::HashSet::new();
        let mut npc_cells = BTreeSet::new();
        let mut npc_scope = None;
        for draw in npcs {
            cancelled(cancel)?;
            let actor = draw.actor;
            ensure!(
                draw.id.scope.match_id == snapshot.world_id
                    && positive_id(draw.id.scope.generation)
                    && positive_id(draw.id.scope.room_id)
                    && positive_id(draw.id.serial)
                    && draw.id.serial == actor.serial
                    && positive_id(actor.life)
                    && npc_ids.insert(draw.id)
                    && npc_scope.is_none_or(|scope| scope == draw.id.scope)
                    && self.map.standing(actor.cell)
                    && draw.source_frame == draw.sample.source
                    && draw.sample.body.asset.graphics_id == actor.graphic,
                "NPC draw scope/identity/form/geometry mismatch"
            );
            npc_scope = Some(draw.id.scope);
            ensure!(
                !actor.alive
                    || (npc_cells.insert(actor.cell)
                        && !snapshot.actors.iter().any(|p| p.cell == actor.cell)
                        && !objectives
                            .iter()
                            .any(|o| o.blocking && o.cell == actor.cell)),
                "NPC draw alive occupancy collision"
            );
            if let Some(from) = draw.walk_from {
                ensure!(
                    actor.alive
                        && draw.sample.action == super::npc_assets::NpcAction::Walk
                        && draw.source_frame != 0
                        && draw.sample.heading <= 7
                        && self.map.can_step(from, actor.cell)
                        && super::world_map::DIRECTIONS[usize::from(draw.sample.heading)]
                            == (actor.cell.x - from.x, actor.cell.y - from.y),
                    "NPC visual walk origin is not a confirmed qualified step"
                );
            }
            let anchor = projection
                .cell_to_pixel(draw.walk_from.unwrap_or(actor.cell))
                .context("NPC screen coordinate overflow")?;
            let frame = draw.sample.body.frame;
            let top_left = layer_position(anchor, frame)?;
            plans.push(PaintPlan {
                // The final namespace tie-breaker never converts a NPC serial
                // into a participant entity ID, including corpse/player ties.
                key: (anchor.y, anchor.x, draw.id.serial, 1),
                frame,
                shadow: Some(draw.sample.shadow.frame),
                attachment: draw.sample.attachment.as_ref().map(|a| a.frame),
                placement: Placement::Npc(NpcPlacement {
                    visibility: None,
                    id: draw.id,
                    life: actor.life,
                    alive: actor.alive,
                    cell: actor.cell,
                    anchor,
                    top_left,
                    width: frame.width,
                    height: frame.height,
                    graphics_id: actor.graphic,
                    source_frame: draw.source_frame,
                    sprite_name: draw.sample.body.provenance.name.clone(),
                    subframe: draw.sample.body.asset.subframe as usize,
                }),
            });
        }
        // Qualify every extra layer before changing pixels.
        for plan in &plans {
            for layer in [plan.shadow, plan.attachment].into_iter().flatten() {
                layer_position(plan.placement.geometry().0, layer)?;
            }
        }
        if plans.iter().all(|plan| {
            [Some(plan.frame), plan.shadow, plan.attachment]
                .into_iter()
                .flatten()
                .all(|frame| frame.fragments.is_some())
        }) {
            return self.render_native(snapshot, canvas, cancel, projection, plans, roofs, effects);
        }
        #[cfg(not(test))]
        {
            anyhow::bail!("world draw has unqualified flattened-only SPR");
        }
        // Hand-painted unit fixtures have no encoded SPR blocks. This path is
        // TEST ONLY; real resource tests use the native fragment path above.
        #[cfg(test)]
        {
            ensure!(
                effects.is_empty(),
                "effect cannot use flattened-only test terrain"
            );
            plans.sort_by_key(|plan| plan.key);
            canvas.clear([18, 25, 35]);
            let mut visible_tiles = 0;
            let mut tile_scratch = [0; super::tile_pixels::RGBA_BYTES];
            for (index, draw) in self.draws.iter().enumerate() {
                if index % 256 == 0 {
                    cancelled(cancel)?;
                }
                let origin = projection
                    .cell_to_pixel(draw.sector_origin)
                    .context("sector screen coordinate overflow")?;
                // Sector-local cell(0,0) diamond center is (24,768). Every static
                // tile and every actor shares this exact world projection.
                let x = i64::from(origin.x) + i64::from(draw.tile.x) - 24;
                let y = i64::from(origin.y) + i64::from(draw.tile.y) - 768;
                if x >= i64::from(canvas.width())
                    || y >= i64::from(canvas.height())
                    || x + 24 <= 0
                    || y + 24 <= 0
                {
                    continue;
                }
                let tile =
                    &self.tiles[&draw.tile.archive_entry_id][&(draw.tile.sub_index as usize)];
                canvas.blit_rgba(
                    tile.rgba_pixels(&mut tile_scratch),
                    24,
                    24,
                    x.try_into()?,
                    y.try_into()?,
                )?;
                visible_tiles += 1;
            }
            let mut placements = Vec::with_capacity(snapshot.actors.len());
            let mut objective_placements = Vec::with_capacity(objectives.len());
            let mut npc_placements = Vec::with_capacity(npcs.len());
            for plan in plans {
                cancelled(cancel)?;
                let (anchor, top_left) = plan.placement.geometry();
                if let Some(shadow) = plan.shadow {
                    let PixelPoint { x, y } = layer_position(anchor, shadow)?;
                    canvas.blit_native_shadow_rgb555(
                        &shadow.rgba,
                        shadow.width,
                        shadow.height,
                        x,
                        y,
                    )?;
                }
                let frame = plan.frame;
                canvas.blit_rgba(
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    top_left.x,
                    top_left.y,
                )?;
                if let Some(attachment) = plan.attachment {
                    let PixelPoint { x, y } = layer_position(anchor, attachment)?;
                    canvas.blit_screen_rgb555(
                        &attachment.rgba,
                        attachment.width,
                        attachment.height,
                        x,
                        y,
                    )?;
                }
                match plan.placement {
                    Placement::Actor(p) => placements.push(p),
                    Placement::Objective(p) => objective_placements.push(p),
                    Placement::Npc(p) => npc_placements.push(p),
                }
            }
            cancelled(cancel)?;
            Ok(RenderInfo {
                world_id: snapshot.world_id,
                revision: snapshot.revision,
                projection,
                visible_tiles,
                actors: placements,
                objectives: objective_placements,
                npcs: npc_placements,
            })
        }
    }

    fn render_native(
        &self,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        projection: ViewProjection,
        mut plans: Vec<PaintPlan<'_>>,
        roofs: &super::world_roofs::Frame,
        effects: &[super::disintegrate_effect::DisintegrateDraw<'_>],
    ) -> Result<RenderInfo> {
        use super::{
            world_depth::NativeDrawCell,
            world_raster::{Blend, Raster},
        };
        let mut raster = Raster::new(projection, canvas);
        let mut visible_tiles = 0;
        let rows = NativeDrawCell::visible_rows(projection, canvas.height())
            .context("visible native row bounds")?;
        let start = self
            .draws
            .partition_point(|d| d.native.scan_row() < *rows.start());
        let end = self
            .draws
            .partition_point(|d| d.native.scan_row() <= *rows.end());
        for (index, draw) in self.draws[start..end].iter().enumerate() {
            if index % 256 == 0 {
                cancelled(cancel)?;
            }
            let cell = draw.native;
            let position = cell
                .screen_origin(projection)
                .context("map fragment projection overflow")?;
            if i64::from(position.x) >= i64::from(canvas.width())
                || i64::from(position.x) + 24 <= 0
                || i64::from(position.y) >= i64::from(canvas.height())
                || i64::from(position.y) + 24 <= 0
            {
                continue;
            }
            let depth = if draw.tile.layer == -2 {
                None
            } else {
                Some(u8::try_from(draw.tile.layer.max(0))?)
            };
            let tile = &self.tiles[&draw.tile.archive_entry_id][&(draw.tile.sub_index as usize)];
            let shade = roofs.shade(draw.roof_group, matches!(tile.kind() as u8, 6 | 7));
            visible_tiles += usize::from(raster.tile_roof(cell, depth, tile, shade)?);
        }
        // Owner labels and returned placements describe scene geometry, not
        // caller vector positions. Keep raster insertion in its original order:
        // equal-depth native fragments still use the established roster policy.
        let mut placement_order: Vec<usize> = (0..plans.len()).collect();
        placement_order.sort_by_key(|index| plans[*index].key);
        let mut owners = vec![0u16; plans.len()];
        for (rank, index) in placement_order.iter().enumerate() {
            owners[*index] = u16::try_from(rank + 1)?;
        }
        for (index, plan) in plans.iter().enumerate() {
            cancelled(cancel)?;
            let world = projection
                .pixel_to_cell(plan.placement.geometry().0)
                .context("actor native anchor absent")?;
            let origin =
                NativeDrawCell::from_world(world).context("actor native origin overflow")?;
            let owner = owners[index];
            // 4E0200 inserts body, attachment (4E7000), then shadow;
            // equal-depth reverse insertion consequently paints shadow first.
            raster.sprite(origin, plan.frame, owner, Blend::Over)?;
            if let Some(attachment) = plan.attachment {
                raster.sprite(origin, attachment, 0, Blend::Screen)?;
            }
            if let Some(shadow) = plan.shadow {
                raster.sprite(origin, shadow, 0, Blend::Shadow)?;
            }
        }
        // Modern cross-actor policy: prepared events follow roster insertion;
        // the native timer's history-dependent ordering is not reconstructed.
        // Within each effect, original main/clothes source insertion is retained.
        for effect in effects {
            effect.enqueue(&mut raster)?;
        }
        let masks = raster.paint(canvas, cancel, plans.len())?;
        let mut actors = Vec::new();
        let mut objectives = Vec::new();
        let mut npcs = Vec::new();
        for (index, mask) in placement_order.into_iter().zip(masks) {
            let plan = &mut plans[index];
            match &mut plan.placement {
                Placement::Actor(p) => {
                    p.visibility = Some(mask);
                    actors.push(p.clone());
                }
                Placement::Objective(p) => {
                    p.visibility = Some(mask);
                    objectives.push(p.clone());
                }
                Placement::Npc(p) => {
                    p.visibility = Some(mask);
                    npcs.push(p.clone());
                }
            }
        }
        cancelled(cancel)?;
        Ok(RenderInfo {
            world_id: snapshot.world_id,
            revision: snapshot.revision,
            projection,
            visible_tiles,
            actors,
            objectives,
            npcs,
        })
    }
}

fn layer_position(anchor: PixelPoint, frame: &super::sprite::SpriteFrame) -> Result<PixelPoint> {
    let shift = if frame.fragments.is_some() {
        (24, 12)
    } else {
        (0, 0)
    };
    Ok(PixelPoint {
        x: anchor
            .x
            .checked_add(frame.x_offset)
            .and_then(|x| x.checked_sub(shift.0))
            .context("layer X overflow")?,
        y: anchor
            .y
            .checked_add(frame.y_offset)
            .and_then(|y| y.checked_sub(shift.1))
            .context("layer Y overflow")?,
    })
}
enum Placement {
    Actor(ActorPlacement),
    Objective(ObjectivePlacement),
    Npc(NpcPlacement),
}
impl Placement {
    fn geometry(&self) -> (PixelPoint, PixelPoint) {
        match self {
            Self::Actor(p) | Self::Objective(p) => (p.anchor, p.top_left),
            Self::Npc(p) => (p.anchor, p.top_left),
        }
    }
}
struct PaintPlan<'a> {
    #[cfg_attr(not(test), allow(dead_code))]
    key: (i32, i32, u64, u8),
    frame: &'a super::sprite::SpriteFrame,
    shadow: Option<&'a super::sprite::SpriteFrame>,
    attachment: Option<&'a super::sprite::SpriteFrame>,
    placement: Placement,
}
pub(crate) struct NpcDraw<'a> {
    pub id: super::npc_protocol::Id,
    pub actor: NpcBody,
    pub source_frame: usize,
    pub walk_from: Option<WorldCell>,
    pub sample: super::npc_assets::NpcFrame<'a>,
}
/// Protocol-independent, already confirmed body geometry. Rendering does not
/// need a legacy NPC wire actor or its combat/roster revision domain.
#[derive(Clone, Copy)]
pub(crate) struct NpcBody {
    pub serial: u64,
    pub life: u64,
    pub alive: bool,
    pub cell: WorldCell,
    pub graphic: u32,
}
impl From<&super::npc_protocol::Actor> for NpcBody {
    fn from(a: &super::npc_protocol::Actor) -> Self {
        Self {
            serial: a.serial,
            life: a.life,
            alive: a.alive(),
            cell: a.cell,
            graphic: a.graphic,
        }
    }
}
/// NPC geometry carries the full match/generation/serial AND life. It is never
/// accepted by a player-target API or the player inventory/stat UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NpcPlacement {
    pub visibility: Option<super::world_raster::DrawVisibility>,
    pub id: super::npc_protocol::Id,
    pub life: u64,
    pub alive: bool,
    pub cell: WorldCell,
    pub anchor: PixelPoint,
    pub top_left: PixelPoint,
    pub width: u32,
    pub height: u32,
    pub graphics_id: u32,
    pub source_frame: usize,
    pub sprite_name: String,
    pub subframe: usize,
}

pub struct ActorOverride<'a> {
    pub entity_id: u64,
    pub frame: &'a super::sprite::SpriteFrame,
    pub graphics_id: u32,
    pub source_frame: usize,
    pub sprite_name: &'a str,
    pub subframe: usize,
}
/// Resource-minted visual placement. Authoritative ActorPlacement.cell remains
/// the confirmed destination; neither this anchor nor its camera grants MOVE.
pub(crate) struct PlayerWalkPlacement {
    world: u64,
    revision: u64,
    entity: u64,
    form: CombatAppearance,
    logical: u32,
    heading: u8,
    from: WorldCell,
    to: WorldCell,
    source: usize,
    terminal: bool,
    offset: (i32, i32),
}
impl PlayerWalkPlacement {
    pub(crate) fn entity(&self) -> u64 {
        self.entity
    }
    pub(crate) fn source(&self) -> usize {
        self.source
    }
}
/// Already-qualified pet bank sample, not an arbitrary player replacement.
pub(crate) struct PetDraw<'a> {
    pub entity_id: u64,
    pub anchor_cell: WorldCell,
    pub sample: super::pet_animation::PetFrameSample<'a>,
}
pub struct ObjectiveDraw<'a> {
    pub entity_id: u64,
    /// Only a living authoritative objective occupies its cell. A ruin is art.
    pub blocking: bool,
    pub cell: WorldCell,
    pub frame: &'a super::sprite::SpriteFrame,
    pub shadow: Option<&'a super::sprite::SpriteFrame>,
    pub graphics_id: u32,
    pub source_frame: usize,
    pub sprite_name: &'a str,
    pub subframe: usize,
}
/// Screen geometry only: contains no source identity or character attributes.
pub type ObjectivePlacement = ActorPlacement;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActorPlacement {
    pub visibility: Option<super::world_raster::DrawVisibility>,
    pub entity_id: u64,
    pub cell: WorldCell,
    pub anchor: PixelPoint,
    pub top_left: PixelPoint,
    pub width: u32,
    pub height: u32,
    pub graphics_id: u32,
    pub source_frame: usize,
    pub sprite_name: String,
    pub subframe: usize,
}

pub struct RenderInfo {
    pub world_id: u64,
    pub revision: u64,
    pub projection: ViewProjection,
    pub visible_tiles: usize,
    pub actors: Vec<ActorPlacement>,
    pub objectives: Vec<ObjectivePlacement>,
    pub npcs: Vec<NpcPlacement>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::modern_lito::{
        resources::SharedMapSectorKey,
        sprite::SpriteFrame,
        tile::{ClassicTileType, DecodedTile},
        world_map::{SectorBinding, SectorInput},
    };

    #[test]
    fn only_observed_walk_effects_are_retained_without_relabeling_or_execution() {
        let mut sounds = BTreeSet::new();
        let mut effects = Vec::new();
        for class in 0..7 {
            for sex in 0..2 {
                let appearance = BaseAppearance { class, sex };
                for (index, id) in [478, 479].into_iter().enumerate() {
                    retain_base_cue(
                        appearance,
                        BaseAction::Walk,
                        index * 2,
                        index as u64 * 8,
                        &FrameEvent::Effect(id),
                        &mut sounds,
                        &mut effects,
                    )
                    .unwrap();
                    assert!(retain_base_cue(
                        appearance,
                        BaseAction::Idle,
                        0,
                        0,
                        &FrameEvent::Effect(id),
                        &mut sounds,
                        &mut effects
                    )
                    .is_err());
                }
                for event in [
                    FrameEvent::Effect(477),
                    FrameEvent::Effect(480),
                    FrameEvent::Hit,
                    FrameEvent::Overlay(478),
                    FrameEvent::Skip,
                    FrameEvent::UninterpretedNumeric {
                        marker: b'^',
                        value: 478,
                    },
                ] {
                    assert!(retain_base_cue(
                        appearance,
                        BaseAction::Walk,
                        0,
                        0,
                        &event,
                        &mut sounds,
                        &mut effects
                    )
                    .is_err());
                }
            }
        }
        assert!(sounds.is_empty());
        assert_eq!(effects.len(), 28);
        assert_eq!(
            effects[1],
            UnrenderedBaseEffect {
                appearance: BaseAppearance { class: 0, sex: 0 },
                action: BaseAction::Walk,
                frame_index: 2,
                tick: 8,
                effect_id: 479,
            }
        );
        retain_base_cue(
            BaseAppearance { class: 1, sex: 0 },
            BaseAction::Walk,
            1,
            4,
            &FrameEvent::Sound(123),
            &mut sounds,
            &mut effects,
        )
        .unwrap();
        assert_eq!(sounds, BTreeSet::from([123]));
        assert_eq!(effects.len(), 28);
    }

    fn key() -> SharedMapSectorKey {
        SharedMapSectorKey {
            map_id: 514,
            sector_x: 0x7fff,
            sector_y: 0x8000,
        }
    }
    fn open_map() -> Arc<WorldMap> {
        Arc::new(
            WorldMap::from_sector_bytes(
                vec![SectorInput {
                    key: key(),
                    bytes: vec![0; 49_170],
                }],
                &AtomicBool::new(false),
            )
            .unwrap(),
        )
    }
    fn actor(entity: u64, source: u32, x: i32, y: i32) -> ActorState {
        ActorState {
            kind: super::super::participant::MemberKind::Human,
            entity_id: entity,
            source_id: source,
            object_id: 3333,
            appearance: BaseAppearance { class: 1, sex: 0 },
            weapon_type: 0,
            cell: WorldCell { x, y },
            heading: 0,
            action: BaseAction::Idle,
            action_elapsed: Duration::ZERO,
        }
    }
    fn snapshot(
        map: &WorldMap,
        selected: u64,
        revision: u64,
        actors: Vec<ActorState>,
    ) -> SceneSnapshot {
        SceneSnapshot::new(19, revision, map.canonical_digest(), selected, actors).unwrap()
    }

    #[test]
    fn base_class_sex_maps_to_logical_records_not_one_graphics_bank() {
        for (class, male, female) in [
            (0, 0, 1),
            (1, 61, 48),
            (2, 138, 37),
            (3, 734, 1186),
            (4, 2786, 2796),
            (5, 6658, 6661),
            (6, 6671, 6650),
        ] {
            assert_eq!(
                BaseAppearance { class, sex: 0 }
                    .logical_sprite_id()
                    .unwrap(),
                male
            );
            assert_eq!(
                BaseAppearance { class, sex: 1 }
                    .logical_sprite_id()
                    .unwrap(),
                female
            );
        }
        assert!(BaseAppearance { class: 7, sex: 0 }
            .logical_sprite_id()
            .is_err());
        assert!(BaseAppearance { class: 0, sex: 2 }
            .logical_sprite_id()
            .is_err());
        assert_eq!(BaseAction::Idle.action_key(), ActionKey::new(3));
        assert_eq!(BaseAction::Walk.action_key(), ActionKey::new(0));
    }

    #[test]
    fn native_sprite_hash_priority_only_falls_back_on_missing_member() {
        assert_eq!(sprite_shard("3213-0.spr"), 9);
        assert_eq!(sprite_shard("3225-8.spr"), 4);
        assert_eq!(sprite_shard("6661-15.spr"), 9);
        assert_eq!(
            route_sprite_member(9, Ok(24), || panic!("base must not win collision")).unwrap(),
            (SharedArchiveKind::SpriteShard(9), 24)
        );
        assert_eq!(
            route_sprite_member(
                9,
                Err(ResourceError::MissingEntry("3213-0.spr".into())),
                || Ok(42)
            )
            .unwrap(),
            (SharedArchiveKind::Sprite, 42)
        );
        assert!(matches!(
            route_sprite_member(
                9,
                Err(ResourceError::MalformedIndex("fixture corruption")),
                || panic!("damage is not a missing member")
            ),
            Err(ResourceError::MalformedIndex(_))
        ));
        assert!(matches!(
            route_sprite_member(
                9,
                Err(ResourceError::MissingEntry("3213-0.spr".into())),
                || Err(ResourceError::MissingEntry("3213-0.spr".into()))
            ),
            Err(ResourceError::MissingEntry(_))
        ));
    }

    #[test]
    fn immutable_snapshot_validates_entity_character_occupancy_and_selected_identity() {
        let a = actor(1001, 7, 32724, 32784);
        let b = actor(1002, 42, 32725, 32784); // Same objid, DIFFERENT source is legal.
        let snapshot =
            SceneSnapshot::new(19, 1, [9; 32], 1001, vec![b.clone(), a.clone()]).unwrap();
        assert_eq!(snapshot.actors()[0].entity_id, 1001);
        assert_eq!(
            (
                snapshot.world_id(),
                snapshot.revision(),
                snapshot.selected_entity_id()
            ),
            (19, 1, 1001)
        );
        let make = |actors| SceneSnapshot::new(19, 1, [9; 32], 1001, actors);
        assert!(make(vec![]).is_err());
        assert!(make(vec![a.clone(); 101]).is_err());
        assert!(make(vec![b.clone()]).is_err());
        for invalid in [
            ActorState {
                entity_id: 1001,
                ..b.clone()
            },
            ActorState {
                source_id: 7,
                ..b.clone()
            },
            ActorState {
                cell: a.cell,
                ..b.clone()
            },
            ActorState {
                source_id: 0,
                ..b.clone()
            },
            ActorState {
                object_id: 0,
                ..b.clone()
            },
            ActorState {
                heading: 8,
                ..b.clone()
            },
            ActorState {
                entity_id: u64::MAX,
                ..b.clone()
            },
        ] {
            assert!(make(vec![a.clone(), invalid]).is_err());
        }
        assert!(SceneSnapshot::new(0, 1, [9; 32], 1001, vec![a.clone()]).is_err());
        assert!(SceneSnapshot::new(19, 0, [9; 32], 1001, vec![a.clone()]).is_err());
        assert!(SceneSnapshot::new(19, u64::MAX, [9; 32], 1001, vec![a]).is_err());
    }

    #[test]
    fn compact_tile_budget_charges_retained_capacity_and_row_masks_before_publication() {
        let decoded = super::super::tile::DecodedTile {
            tile_type: super::super::tile::ClassicTileType::RleEven,
            source_offset: 0,
            x_offset: 0,
            y_offset: 0,
            content_width: 24,
            content_height: 24,
            width: 24,
            height: 24,
            rgba: vec![0; 2304],
        };
        let tile = super::super::world_raster::PreparedTile::new(decoded)
            .unwrap()
            .compact_storage()
            .unwrap();
        assert_eq!(tile.retained_bytes(), 72); // Transparent pixels are absent; row masks are not free.
        let mut budget = Budget {
            limits: SceneLimits {
                max_texture_bytes: 2304,
                ..Default::default()
            }
            .validate()
            .unwrap(),
            encoded: 0,
            decoded: 0,
        };
        for _ in 0..32 {
            budget.decoded_entry(tile.retained_bytes()).unwrap();
        }
        assert_eq!(budget.decoded, 2304);
        assert!(budget.decoded_entry(tile.retained_bytes()).is_err());
        assert_eq!(budget.decoded, 2304);
    }
    #[test]
    fn aggregate_budgets_reject_before_entry_read_and_cancel_before_root_open() {
        let limits = SceneLimits {
            max_encoded_bytes: 32,
            max_texture_bytes: 24 * 24 * 4,
            max_draw_tiles: 1,
        }
        .validate()
        .unwrap();
        let mut budget = Budget {
            limits,
            encoded: 0,
            decoded: 0,
        };
        budget.encoded_entry(16).unwrap();
        assert!(budget.encoded_entry(17).is_err());
        assert_eq!(budget.encoded, 16);
        budget.encoded_entry(16).unwrap();
        assert!(budget.encoded_entry(1).is_err());
        assert!(budget.encoded_entry(0).is_err());
        assert!(budget
            .encoded_entry(MAX_SOURCE_ENTRY_BYTES as u32 + 1)
            .is_err());
        budget.decoded_entry(24 * 24 * 4 - 4).unwrap();
        assert_eq!(budget.remaining_pixels().unwrap(), 1);
        assert!(budget.decoded_entry(5).is_err());
        budget.decoded_entry(4).unwrap();
        assert!(budget.remaining_pixels().is_err());
        assert!(SceneLimits {
            max_encoded_bytes: 0,
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(SceneLimits {
            max_texture_bytes: 256 * 1024 * 1024 + 1,
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(WorldSceneAssets::load(
            Path::new("must-not-open-this-root"),
            open_map(),
            &[BaseAppearance { class: 1, sex: 0 }],
            SceneLimits::default(),
            &AtomicBool::new(true)
        )
        .is_err());
    }

    #[test]
    fn standalone_catalog_wrappers_keep_validation_ahead_of_resource_open() {
        use super::super::shared_action_catalog::test_counts;
        test_counts::reset();
        let root = Path::new("must-not-open-this-root");
        let map = open_map();
        let cancel = AtomicBool::new(false);
        let base = BaseAppearance { class: 1, sex: 0 };
        for result in [
            WorldSceneAssets::load(root, map.clone(), &[], SceneLimits::default(), &cancel),
            WorldSceneAssets::load_combat(root, map.clone(), &[], SceneLimits::default(), &cancel),
            WorldSceneAssets::load_combat_armed(
                root,
                map.clone(),
                &[],
                SceneLimits::default(),
                &cancel,
            ),
        ] {
            assert_eq!(
                result.err().expect("invalid count before IO").to_string(),
                "world appearance input count exceeds bounds"
            );
        }
        let error = WorldSceneAssets::load(
            root,
            map.clone(),
            &[base],
            SceneLimits {
                max_encoded_bytes: 0,
                ..SceneLimits::default()
            },
            &cancel,
        )
        .err()
        .unwrap();
        assert_eq!(error.to_string(), "invalid bounded world scene limits");
        let error = WorldSceneAssets::load_combat_armed(
            root,
            map.clone(),
            &[CombatAppearance {
                appearance: BaseAppearance { class: 5, sex: 0 },
                weapon_type: 4,
            }],
            SceneLimits::default(),
            &cancel,
        )
        .err()
        .unwrap();
        assert_eq!(error.to_string(), "unqualified class/weapon shared stance");
        let error = WorldSceneAssets::load_combat(
            root,
            map,
            &[base],
            SceneLimits::default(),
            &AtomicBool::new(true),
        )
        .err()
        .unwrap();
        assert_eq!(error.to_string(), "world scene loading/rendering cancelled");
        assert_eq!(test_counts::snapshot(), test_counts::Counts::default());
    }

    #[test]
    fn shared_catalog_validation_deduplicates_only_exact_appearance_weapon_pairs() {
        let unarmed = CombatAppearance {
            appearance: BaseAppearance { class: 1, sex: 0 },
            weapon_type: 0,
        };
        let sword = CombatAppearance {
            weapon_type: 1,
            ..unarmed
        };
        let (limits, appearances) = WorldSceneAssets::validate_load(
            &[sword, unarmed, sword],
            SceneLimits::default(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(appearances, BTreeSet::from([unarmed, sword]));
        assert_eq!(
            limits.max_encoded_bytes,
            SceneLimits::default().max_encoded_bytes
        );
        assert_eq!(
            limits.max_texture_bytes,
            SceneLimits::default().max_texture_bytes
        );
        assert!(WorldSceneAssets::validate_load(
            &vec![unarmed; MAX_ACTORS + 1],
            SceneLimits::default(),
            &AtomicBool::new(false)
        )
        .is_err());
    }
    #[test]
    fn human_bot_numeric_collision_is_renderable_but_bot_cannot_be_selected() {
        let a = actor(1001, 7, 32724, 32784);
        let mut b = actor(1002, 7, 32725, 32784);
        b.kind = super::super::participant::MemberKind::ServerBot;
        assert_eq!((a.source_id, a.object_id), (b.source_id, b.object_id));
        assert!(SceneSnapshot::new(19, 1, [9; 32], 1001, vec![a.clone(), b.clone()]).is_ok());
        assert!(SceneSnapshot::new(19, 1, [9; 32], 1002, vec![a.clone(), b.clone()]).is_err());
        b.kind = super::super::participant::MemberKind::Human;
        assert!(SceneSnapshot::new(19, 1, [9; 32], 1001, vec![a, b]).is_err());
    }

    #[test]
    fn source_ticks_loop_without_zero_duration_frames_or_time_proportional_work() {
        let animation = AnimationAssets {
            info: AnimationInfo {
                appearance: BaseAppearance { class: 1, sex: 0 },
                logical_sprite_id: 61,
                graphics_id: 3213,
                action: BaseAction::Walk,
                heading: 0,
                frame_count: 3,
                frames_per_second: 16,
                total_ticks: 5,
            },
            ends: vec![0, 2, 5],
            frames: vec![],
        };
        assert_eq!(animation.source_frame(Duration::ZERO), 1);
        assert_eq!(animation.source_frame(Duration::from_millis(124)), 1);
        assert_eq!(animation.source_frame(Duration::from_millis(125)), 2);
        assert_eq!(animation.source_frame(Duration::from_micros(312_500)), 1);
        assert!(animation.source_frame(Duration::MAX) < 3);
    }

    pub(crate) fn synthetic_assets() -> WorldSceneAssets {
        let map = open_map();
        let (ox, oy) = key().world_origin();
        let (x, y) = crate::modern_lito::scene::project_half_tile(40, 20);
        let tile = DecodedTile {
            tile_type: ClassicTileType::RleEven,
            source_offset: 0,
            x_offset: 0,
            y_offset: 0,
            content_width: 24,
            content_height: 24,
            width: 24,
            height: 24,
            rgba: [0, 200, 0, 255].repeat(24 * 24),
        };
        let mut sprites = BTreeMap::new();
        let mut animations = BTreeMap::new();
        let appearance = BaseAppearance { class: 1, sex: 0 };
        for action in [BaseAction::Idle, BaseAction::Walk] {
            for heading in 0..8 {
                let image = if action == BaseAction::Idle { 8 } else { 0 } + heading;
                let name = format!("3213-{image}.spr");
                let frame = |color: [u8; 4]| SpriteFrame {
                    fragments: None,
                    x_offset: -2,
                    y_offset: -4,
                    width: 5,
                    height: 5,
                    frame_type: 0,
                    unknown1: 0,
                    unknown2: 0,
                    rgba: color.repeat(25),
                };
                sprites.insert(
                    name.clone(),
                    SpriteAsset {
                        source: SpriteSource {
                            archive: SharedArchiveKind::Sprite,
                            name: name.clone(),
                            encoded_bytes: 1,
                            sha256: [1; 32],
                        },
                        decoded: DecodedSprite {
                            frames: vec![
                                frame([200, heading, 0, 255]),
                                frame([0, heading, 200, 255]),
                            ],
                            has_palette: false,
                            block_table_metadata: 0,
                        },
                    },
                );
                animations.insert(
                    (
                        CombatAppearance {
                            appearance,
                            weapon_type: 0,
                        },
                        action,
                        heading,
                    ),
                    AnimationAssets {
                        info: AnimationInfo {
                            appearance,
                            logical_sprite_id: 61,
                            graphics_id: 3213,
                            action,
                            heading,
                            frame_count: 2,
                            frames_per_second: 10,
                            total_ticks: 2,
                        },
                        ends: vec![1, 2],
                        frames: vec![
                            FrameBinding {
                                sprite: name.clone(),
                                subframe: 0,
                            },
                            FrameBinding {
                                sprite: name,
                                subframe: 1,
                            },
                        ],
                    },
                );
            }
        }
        WorldSceneAssets {
            roofs: super::super::world_roofs::RoofMap::default(),
            map,
            draws: vec![WorldDraw::new(
                DrawTile {
                    half_x: 40,
                    half_y: 20,
                    archive_entry_id: 0,
                    sub_index: 0,
                    x,
                    y,
                    layer: -2,
                },
                WorldCell { x: ox, y: oy },
            )
            .unwrap()],
            tiles: BTreeMap::from([(
                0,
                BTreeMap::from([(
                    0,
                    super::super::world_raster::PreparedTile::new(tile).unwrap(),
                )]),
            )]),
            sprites,
            animations,
            combat: BTreeMap::new(),
            walks: BTreeMap::new(),
            limitations: SceneLimitations {
                opacity_records: 0,
                sprite_effects: 0,
                portals: 0,
                actor_object_occlusion_unimplemented: true,
                actor_dependent_roofs_unimplemented: true,
                native_roof_timers_and_aliases_unverified: true,
                native_actor_update_order_unverified: true,
                weapon_and_transformation_art_unimplemented: true,
                native_actor_foot_pivot_unverified: true,
                unplayed_sound_ids: vec![],
                unrendered_base_effect_ids: vec![],
                unrendered_base_effects: vec![],
            },
            encoded_bytes: 0,
            texture_bytes: 0,
        }
    }

    #[test]
    fn native_objective_input_order_keeps_depth_ordered_placements_and_hit_labels() {
        use super::super::sprite::{CompactSpriteBlock, SpriteBlockReference, SpriteFrameFragments};
        let mut assets = synthetic_assets();
        for sprite in assets.sprites.values_mut() {
            for frame in &mut sprite.decoded.frames {
                frame.width = 24;
                frame.height = 24;
                frame.x_offset = 0;
                frame.y_offset = 0;
                frame.rgba = [240, 40, 30, 255].repeat(576);
                frame.fragments = Some(SpriteFrameFragments {
                    references: vec![SpriteBlockReference {
                        half_x: 0, y: 0, depth: 10, block_id: 0,
                    }],
                    atlas: Arc::new(vec![CompactSpriteBlock::test_rgba(&frame.rgba)]),
                });
            }
        }
        let (ox, oy) = key().world_origin();
        let scene = snapshot(&assets.map, 1, 1, vec![
            actor(1, 7, ox + 20, oy + 20),
            actor(2, 8, ox + 19, oy + 19),
        ]);
        let frame = &assets.sprites.values().next().unwrap().decoded.frames[0];
        let objective = |entity_id, x, y| ObjectiveDraw {
            blocking: true, entity_id, cell: WorldCell { x, y }, frame,
            shadow: None, graphics_id: 10, source_frame: 0,
            sprite_name: "synthetic-fragment.spr", subframe: 0,
        };
        let mut draws = vec![objective(3, ox + 21, oy + 21), objective(4, ox + 24, oy + 24)];
        let cancel = AtomicBool::new(false);
        let mut first = Canvas::new(512, 256).unwrap();
        let before = assets.render_into_objectives(&scene, &mut first, &cancel, &[], &draws).unwrap();
        draws.reverse();
        let mut second = Canvas::new(512, 256).unwrap();
        let after = assets.render_into_objectives(&scene, &mut second, &cancel, &[], &draws).unwrap();
        assert_eq!(first.rgba(), second.rgba());
        assert_eq!(before.actors, after.actors);
        assert_eq!(before.objectives, after.objectives);
        assert_eq!(before.actors.iter().map(|p| p.entity_id).collect::<Vec<_>>(), [2, 1]);
        assert_eq!(before.objectives.iter().map(|p| p.entity_id).collect::<Vec<_>>(), [3, 4]);
        for placement in before.actors.iter().chain(&before.objectives) {
            assert!(placement.top_left.x >= 0 && placement.top_left.y >= 0);
            assert!(placement.top_left.x + placement.width as i32 <= 512);
            assert!(placement.top_left.y + placement.height as i32 <= 256);
            let mask = placement.visibility.as_ref().unwrap();
            assert!((0..256).any(|y| (0..512).any(|x| mask.contains(x, y))));
        }
    }

    #[test]
    fn actual_layer5_pipeline_fades_only_selected_actor_group_and_keeps_hits() {
        use super::super::{
            sprite::{CompactSpriteBlock, SpriteBlockReference, SpriteFrameFragments},
            world_roofs::{Animation, Group},
        };
        let mut assets = synthetic_assets();
        let mut raw = vec![0; 32768 + 2 + 16384];
        raw.extend_from_slice(&1i32.to_le_bytes());
        raw.extend_from_slice(&4444i16.to_le_bytes());
        raw.extend_from_slice(&1u16.to_le_bytes());
        raw.extend_from_slice(&[40, 20, 20, 0, 0, 0, 0]);
        raw.extend_from_slice(&1i32.to_le_bytes());
        raw.extend_from_slice(&[40, 20, 0, 0, 1]);
        assets.map = Arc::new(
            WorldMap::from_sector_bytes(
                vec![SectorInput {
                    key: key(),
                    bytes: raw,
                }],
                &AtomicBool::new(false),
            )
            .unwrap(),
        );
        assets.roofs = super::super::world_roofs::RoofMap::new(&assets.map).unwrap();
        assets.draws[0].tile.layer = 20;
        assets.draws[0].roof_group = Some(Group(key().sector_x, key().sector_y, 0));
        for sprite in assets.sprites.values_mut() {
            for f in &mut sprite.decoded.frames {
                f.width = 24;
                f.height = 24;
                f.x_offset = 0;
                f.y_offset = 0;
                f.fragments = Some(SpriteFrameFragments {
                    references: vec![SpriteBlockReference {
                        half_x: 0,
                        y: 0,
                        depth: 10,
                        block_id: 0,
                    }],
                    atlas: Arc::new(vec![CompactSpriteBlock::test_rgba(
                        &[255, 0, 0, 255].repeat(576),
                    )]),
                });
            }
        }
        let (ox, oy) = key().world_origin();
        let inside = actor(1, 7, ox + 20, oy + 20);
        let outside = actor(2, 8, ox + 21, oy + 20);
        let scene = snapshot(&assets.map, 1, 1, vec![inside.clone(), outside.clone()]);
        let mut canvas = Canvas::new(100, 100).unwrap();
        let cancel = AtomicBool::new(false);
        let t = std::time::Instant::now();
        let mut animation = Animation::default();
        let f = assets.sample_roofs(&mut animation, &scene, t).unwrap();
        let info = assets
            .render_into_npcs_roofs(&scene, &mut canvas, &cancel, &[], &[], &[], Some(&f))
            .unwrap();
        assert_eq!(pixel(&canvas, 26, 38), [0, 200, 0, 255]);
        assert!(!info.actors.iter().find(|p| p.entity_id == 1).expect("inside actor absent").visibility.as_ref().unwrap().contains(26, 38));
        let f = assets
            .sample_roofs(&mut animation, &scene, t + Duration::from_millis(100))
            .unwrap();
        let info = assets
            .render_into_npcs_roofs(&scene, &mut canvas, &cancel, &[], &[], &[], Some(&f))
            .unwrap();
        assert!(pixel(&canvas, 26, 38)[0] > 0);
        assert!(pixel(&canvas, 26, 38)[1] > 0);
        assert!(info.actors.iter().find(|p| p.entity_id == 1).expect("inside actor absent").visibility.as_ref().unwrap().contains(26, 38));
        let f = assets
            .sample_roofs(&mut animation, &scene, t + Duration::from_millis(400))
            .unwrap();
        assets
            .render_into_npcs_roofs(&scene, &mut canvas, &cancel, &[], &[], &[], Some(&f))
            .unwrap();
        assert_eq!(pixel(&canvas, 26, 38), [255, 0, 0, 255]);
        // Same scene and stationary inside actor, but another human selected:
        // the old local actor's roof must not stay hidden across ownership.
        let scene = snapshot(&assets.map, 2, 2, vec![inside, outside]);
        let f = assets
            .sample_roofs(&mut animation, &scene, t + Duration::from_millis(450))
            .unwrap();
        let info = assets
            .render_into_npcs_roofs(&scene, &mut canvas, &cancel, &[], &[], &[], Some(&f))
            .unwrap();
        let p = info.actors.iter().find(|p| p.entity_id == 1).expect("inside actor absent");
        assert_eq!(
            pixel(&canvas, p.anchor.x as usize - 24, p.anchor.y as usize - 12),
            [0, 200, 0, 255]
        );
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; complete Honor raw roof cache vs independent C# oracle and budget rejection"]
    fn real_honor_effect_roof_cache_matches_csharp_and_counts_full_budget() {
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
        );
        let cancel = AtomicBool::new(false);
        let mut inputs = Vec::new();
        for (sx, sy, hash) in [
            (
                0x7fff,
                0x8000,
                "d35e6f1da1c1012e68f05cdc0b6b830040fea41fdb13f24f4596a62dfeab0db3",
            ),
            (
                0x7fff,
                0x8001,
                "147e88b94f0f0f4a3277a1e9166436e534ee2f455cd8cdbe2da39778f0cca931",
            ),
            (
                0x8000,
                0x8000,
                "56bf52953d3483f3ba9399fac9b935b2059486713ee42412626bcd5a7ecb5899",
            ),
            (
                0x8000,
                0x8001,
                "77c86ad27d63e457c32aba1502ce95d397b00590e737b20bad243a035b617d9f",
            ),
            (
                0x8001,
                0x8000,
                "409856e787efd3dea1cd9f7a9c450d2778d3c877cf83ec51423e91b5ba39776a",
            ),
            (
                0x8001,
                0x8001,
                "121e8853304cc849abbdd8e8e696d32ed1b0bd82ed4d538155f8891c9445591a",
            ),
        ] {
            let key = SharedMapSectorKey {
                map_id: 532,
                sector_x: sx,
                sector_y: sy,
            };
            let bytes = super::super::resources::read_shared_map_sector_with_limit(
                &root,
                key,
                super::super::resources::MAX_SHARED_MAP_SECTOR_BYTES,
            )
            .unwrap();
            assert_eq!(format!("{:x}", sha2::Sha256::digest(&bytes)), hash);
            inputs.push(SectorInput { key, bytes });
        }
        let map = Arc::new(WorldMap::from_sector_bytes(inputs, &cancel).unwrap());
        let appearances = [BaseAppearance { class: 1, sex: 0 }];
        let assets = WorldSceneAssets::load(
            &root,
            map.clone(),
            &appearances,
            SceneLimits::default(),
            &cancel,
        )
        .unwrap();
        let mut canonical = Vec::new();
        let mut kinds = BTreeMap::new();
        let controlled: BTreeSet<_> = assets
            .draws
            .iter()
            .filter(|d| d.roof_group.is_some_and(|g| assets.roofs.controls(g)))
            .map(|d| (d.tile.archive_entry_id, d.tile.sub_index as usize))
            .collect();
        for (&id, set) in &assets.tiles {
            for (&sub, tile) in set {
                assert_eq!(
                    tile.roof_raw_rgba().is_some(),
                    controlled.contains(&(id, sub)) && tile.kind() as u8 & 0x30 != 0
                );
                if let Some(raw) = tile.roof_raw_rgba() {
                    assert_eq!(id, 2959);
                    canonical.extend_from_slice(&id.to_le_bytes());
                    canonical.extend_from_slice(&[sub as u8, tile.kind() as u8]);
                    canonical.extend_from_slice(&raw);
                    *kinds.entry(tile.kind() as u8).or_insert(0usize) += 1;
                }
            }
        }
        assert_eq!(kinds, BTreeMap::from([(18, 2), (19, 1)]));
        assert_eq!(canonical.len(), 6924);
        assert_eq!(
            format!("{:x}", sha2::Sha256::digest(&canonical)),
            "732bc941775effd8e95c84afb82c4f6a83f4d2c827d9e40b1a61ef75e82f7b97"
        );
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        assert_eq!(
            format!(
                "{:x}",
                sha2::Sha256::digest(archive.read_entry("2959.til").unwrap())
            ),
            "f5330243d89555102c97401bbb5c905fe23a2e2a97752e252685b1f4f128daab"
        );
        let needed = assets.texture_bytes;
        drop(assets);
        let error = WorldSceneAssets::load(
            &root,
            map,
            &appearances,
            SceneLimits {
                max_texture_bytes: needed - 1,
                ..Default::default()
            },
            &cancel,
        )
        .err()
        .expect("one-byte-under budget must reject");
        assert!(format!("{error:#}").contains("budget"), "{error:#}");
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; real S32 Layer5 enter/fade/restore and connected route; optional owned PNG directory"]
    fn real_shared_roofs_fade_and_restore_without_archive_reads_during_render() {
        use super::super::world_roofs::{Animation, Group, Shade};
        use std::collections::VecDeque;
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
        );
        let key = SharedMapSectorKey {
            map_id: 514,
            sector_x: 0x7ffd,
            sector_y: 0x7fff,
        };
        let cancel = AtomicBool::new(false);
        let bytes = super::super::resources::read_shared_map_sector_with_limit(
            &root,
            key,
            super::super::resources::MAX_SHARED_MAP_SECTOR_BYTES,
        )
        .unwrap();
        assert_eq!(
            format!("{:x}", sha2::Sha256::digest(&bytes)),
            "ac3e658f59e20bc6e5fcd4eac7cd8f08a405882ad990dd1aa82e4c5c5002c83c"
        );
        let map = Arc::new(
            WorldMap::from_sector_bytes(vec![SectorInput { key, bytes }], &cancel).unwrap(),
        );
        let assets = WorldSceneAssets::load(
            &root,
            map.clone(),
            &[BaseAppearance { class: 1, sex: 0 }],
            SceneLimits::default(),
            &cancel,
        )
        .unwrap();
        let (ox, oy) = key.world_origin();
        let sector = map.sector(key).unwrap();
        let mut canvas = Canvas::new(800, 600).unwrap();
        for kind in [0, 1] {
            let record = sector
                .opacity
                .iter()
                .find(|r| {
                    r.kind == kind
                        && r.x < 128
                        && r.y < 64
                        && r.x % 2 == 0
                        && map.standing(WorldCell {
                            x: ox + i32::from(r.x) / 2,
                            y: oy + i32::from(r.y),
                        })
                })
                .unwrap();
            let inside = WorldCell {
                x: ox + i32::from(record.x) / 2,
                y: oy + i32::from(record.y),
            };
            let group = Group(key.sector_x, key.sector_y, record.object_index);
            let mut queue = VecDeque::from([inside]);
            let mut parents = BTreeMap::from([(inside, inside)]);
            let outside = loop {
                let p = queue
                    .pop_front()
                    .expect("real roof must have a connected exit");
                if assets.roofs.settled(p).unwrap().shade(Some(group), false) == Shade::Native {
                    break p;
                }
                for (dx, dy) in super::super::world_map::DIRECTIONS {
                    let next = WorldCell {
                        x: p.x + dx,
                        y: p.y + dy,
                    };
                    if !parents.contains_key(&next) && map.can_step(p, next) {
                        parents.insert(next, p);
                        queue.push_back(next);
                    }
                }
                assert!(parents.len() <= 4096);
            };
            let mut route = vec![outside];
            while *route.last().unwrap() != inside {
                route.push(parents[route.last().unwrap()]);
            }
            assert!(route.len() > 1);
            assert!(route.windows(2).all(|pair| map.can_step(pair[0], pair[1])));
            let scene = snapshot(&map, 1, 1, vec![actor(1, 7, inside.x, inside.y)]);
            let t = std::time::Instant::now();
            let mut animation = Animation::default();
            let mut pixels = Vec::new();
            for (label, ms) in [("enter", 0), ("fading", 100), ("settled", 500)] {
                let frame = assets
                    .sample_roofs(&mut animation, &scene, t + Duration::from_millis(ms))
                    .unwrap();
                let info = assets
                    .render_into_npcs_roofs(
                        &scene,
                        &mut canvas,
                        &cancel,
                        &[],
                        &[],
                        &[],
                        Some(&frame),
                    )
                    .unwrap();
                assert_eq!(info.actors[0].cell, inside);
                assert!(info.visible_tiles > 0);
                pixels.push(canvas.rgba().to_vec());
                if let Some(dir) = std::env::var_os("LITO_MODERN_ROOF_CAPTURE_DIR") {
                    let dir = std::path::PathBuf::from(dir);
                    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("target")
                        .canonicalize()
                        .unwrap();
                    assert!(dir.is_absolute() && dir.canonicalize().unwrap().starts_with(&target));
                    let file = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(dir.join(format!("roof-kind{kind}-{label}.png")))
                        .unwrap();
                    let mut encoder = png::Encoder::new(file, 800, 600);
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    encoder
                        .write_header()
                        .unwrap()
                        .write_image_data(canvas.rgba())
                        .unwrap();
                }
            }
            assert_ne!(pixels[0], pixels[1]);
            assert_ne!(pixels[1], pixels[2]);
            assets.render_into(&scene, &mut canvas, &cancel).unwrap();
            assert_eq!(canvas.rgba(), pixels[2]);
            // Follow the verified static exit; no fictitious through-wall path.
            // This is presentation/geometry evidence, not a server move test.
            let mut last = None;
            for (i, p) in route.iter().rev().enumerate() {
                let s = snapshot(&map, 1, i as u64 + 2, vec![actor(1, 7, p.x, p.y)]);
                let f = assets
                    .sample_roofs(
                        &mut animation,
                        &s,
                        t + Duration::from_millis(600 + i as u64 * 100),
                    )
                    .unwrap();
                last = Some((s, f));
            }
            let (s, _) = last.unwrap();
            let f = assets
                .sample_roofs(&mut animation, &s, t + Duration::from_secs(3600))
                .unwrap();
            assert_eq!(f.shade(Some(group), false), Shade::Native);
            assets
                .render_into_npcs_roofs(&s, &mut canvas, &cancel, &[], &[], &[], Some(&f))
                .unwrap();
            let restored = canvas.rgba().to_vec();
            assets.render_into(&s, &mut canvas, &cancel).unwrap();
            assert_eq!(canvas.rgba(), restored);
            eprintln!("shared roof kind{kind} group{} inside={},{} exit={},{} connectedSteps={} rgba={:x}/{:x}/{:x}",record.object_index,inside.x,inside.y,outside.x,outside.y,route.len()-1,sha2::Sha256::digest(&pixels[0]),sha2::Sha256::digest(&pixels[1]),sha2::Sha256::digest(&pixels[2]));
        }
    }

    fn pixel(canvas: &Canvas, x: usize, y: usize) -> &[u8] {
        let offset = (y * canvas.width() as usize + x) * 4;
        &canvas.rgba()[offset..offset + 4]
    }
    #[test]
    fn npc_draws_have_separate_ids_and_share_player_depth_with_body_linked_attachment() {
        let assets = synthetic_assets();
        let (ox, oy) = key().world_origin();
        let pc = actor(1, 7, ox + 20, oy + 20);
        let scene = snapshot(&assets.map, 1, 1, vec![pc.clone()]);
        let mut n = super::super::npc_protocol::tests::read_reply()
            .frame
            .unwrap()
            .actors
            .remove(0);
        n.serial = 1;
        n.graphic = 1578;
        n.cell = WorldCell {
            x: pc.cell.x,
            y: pc.cell.y - 1,
        };
        let id = super::super::npc_protocol::Id {
            scope: super::super::npc_protocol::Scope {
                room_id: 1,
                match_id: 19,
                generation: 2,
            },
            serial: 1,
        };
        let npc_assets = super::super::npc_assets::tests::presentation_fixture(
            super::super::npc_assets::NpcAppearance::new(1578).unwrap(),
        );
        let mut body = SpriteFrame {
            fragments: None,
            x_offset: 23,
            y_offset: 12,
            width: 1,
            height: 1,
            frame_type: 0,
            unknown1: 0,
            unknown2: 0,
            rgba: vec![24, 48, 96, 255],
        };
        let mut canvas = Canvas::new(100, 100).unwrap();
        let mut draw = NpcDraw {
            walk_from: None,
            id,
            actor: (&n).into(),
            source_frame: 1,
            sample: npc_assets
                .frame(
                    super::super::npc_assets::NpcAppearance::new(1578).unwrap(),
                    super::super::npc_assets::NpcAction::Attack,
                    0,
                    1,
                )
                .unwrap(),
        };
        // Adjacent rear NPC's layers land on the front PC's body. If NPCs were
        // appended as an overlay this pixel would incorrectly become NPC blue.
        draw.sample.body.frame = &body;
        draw.sample.shadow.frame = &body;
        draw.sample.attachment.as_mut().unwrap().frame = &body;
        let info = assets
            .render_into_npcs(
                &scene,
                &mut canvas,
                &AtomicBool::new(false),
                &[],
                &[],
                &[draw],
            )
            .unwrap();
        assert_eq!(info.actors[0].entity_id, info.npcs[0].id.serial);
        assert_eq!(info.npcs[0].id, id);
        assert_eq!(info.npcs[0].life, n.life);
        assert_eq!(pixel(&canvas, 49, 50), &[200, 0, 0, 255]);
        // Front NPC paints after PC. Each attachment uses its own offset and
        // screen blend; no attachment has a separate target/hit rectangle.
        n.cell.y = pc.cell.y + 1;
        body.x_offset = -25;
        body.y_offset = -12;
        let mut draw = NpcDraw {
            walk_from: None,
            id,
            actor: (&n).into(),
            source_frame: 1,
            sample: npc_assets
                .frame(
                    super::super::npc_assets::NpcAppearance::new(1578).unwrap(),
                    super::super::npc_assets::NpcAction::Attack,
                    0,
                    1,
                )
                .unwrap(),
        };
        draw.sample.body.frame = &body;
        draw.sample.shadow.frame = &body;
        draw.sample.attachment.as_mut().unwrap().frame = &body;
        let info = assets
            .render_into_npcs(
                &scene,
                &mut canvas,
                &AtomicBool::new(false),
                &[],
                &[],
                &[draw],
            )
            .unwrap();
        assert_eq!(info.actors.len(), 1);
        assert_eq!(info.npcs.len(), 1);
        assert!(info.objectives.is_empty());
        let mut expected = Canvas::new(1, 1).unwrap();
        expected.clear([24, 48, 96]);
        expected.blit_screen_rgb555(&body.rgba, 1, 1, 0, 0).unwrap();
        assert_eq!(pixel(&canvas, 49, 50), expected.rgba());
    }
    #[test]
    fn npc_invalid_scope_or_layer_leaves_surface_unchanged_and_empty_roster_removes_hitboxes() {
        let assets = synthetic_assets();
        let (ox, oy) = key().world_origin();
        let scene = snapshot(&assets.map, 1, 1, vec![actor(1, 7, ox + 20, oy + 20)]);
        let mut n = super::super::npc_protocol::tests::read_reply()
            .frame
            .unwrap()
            .actors
            .remove(0);
        n.cell = WorldCell {
            x: ox + 21,
            y: oy + 20,
        };
        let a = super::super::npc_assets::NpcAppearance::new(n.graphic).unwrap();
        let npc_assets = super::super::npc_assets::tests::presentation_fixture(a);
        let mut canvas = Canvas::new(100, 100).unwrap();
        canvas.clear([7, 8, 9]);
        let before = canvas.rgba().to_vec();
        let bad = NpcDraw {
            walk_from: None,
            id: super::super::npc_protocol::Id {
                scope: super::super::npc_protocol::Scope {
                    room_id: 1,
                    match_id: 20,
                    generation: 2,
                },
                serial: n.serial,
            },
            actor: (&n).into(),
            source_frame: 0,
            sample: npc_assets
                .frame(a, super::super::npc_assets::NpcAction::Breath, 0, 0)
                .unwrap(),
        };
        assert!(assets
            .render_into_npcs(
                &scene,
                &mut canvas,
                &AtomicBool::new(false),
                &[],
                &[],
                &[bad]
            )
            .is_err());
        assert_eq!(canvas.rgba(), before);
        let info = assets
            .render_into_npcs(&scene, &mut canvas, &AtomicBool::new(false), &[], &[], &[])
            .unwrap();
        assert!(info.npcs.is_empty());
        assert_eq!(info.actors.len(), 1);
    }

    #[test]
    fn npc_walk_render_origin_moves_all_layers_without_changing_authoritative_cell() {
        use super::super::npc_assets::{NpcAction, NpcAppearance};
        let assets = synthetic_assets();
        let (ox, oy) = key().world_origin();
        let scene = snapshot(&assets.map, 1, 1, vec![actor(1, 7, ox + 20, oy + 20)]);
        let cancel = AtomicBool::new(false);
        let mut canvas = Canvas::new(100, 100).unwrap();
        let from = WorldCell {
            x: ox + 22,
            y: oy + 22,
        };
        for form in NpcAppearance::ALL {
            let bank = super::super::npc_assets::tests::presentation_fixture(form);
            for (heading, (dx, dy)) in super::super::world_map::DIRECTIONS.into_iter().enumerate() {
                let to = WorldCell {
                    x: from.x + dx,
                    y: from.y + dy,
                };
                let mut n = super::super::npc_protocol::tests::read_reply()
                    .frame
                    .unwrap()
                    .actors
                    .remove(0);
                n.cell = to;
                n.graphic = form.graphics_id();
                let id = super::super::npc_protocol::Id {
                    scope: super::super::npc_protocol::Scope {
                        room_id: 1,
                        match_id: scene.world_id,
                        generation: 1,
                    },
                    serial: n.serial,
                };
                for source in [1, 0] {
                    let draw = NpcDraw {
                        id,
                        actor: (&n).into(),
                        source_frame: source,
                        walk_from: (source != 0).then_some(from),
                        sample: bank
                            .frame(form, NpcAction::Walk, heading as u8, source)
                            .unwrap(),
                    };
                    let info = assets
                        .render_into_npcs(&scene, &mut canvas, &cancel, &[], &[], &[draw])
                        .unwrap();
                    assert_eq!(info.npcs[0].cell, to);
                    assert_eq!(
                        info.npcs[0].anchor,
                        info.projection
                            .cell_to_pixel(if source == 0 { to } else { from })
                            .unwrap()
                    );
                }
                for bad in 0..5 {
                    let mut draw = NpcDraw {
                        id,
                        actor: (&n).into(),
                        source_frame: 1,
                        walk_from: Some(from),
                        sample: bank.frame(form, NpcAction::Walk, heading as u8, 1).unwrap(),
                    };
                    match bad {
                        0 => draw.walk_from = Some(to),
                        1 => {
                            draw.sample = bank
                                .frame(form, NpcAction::Attack, heading as u8, 1)
                                .unwrap()
                        }
                        2 => {
                            draw.sample = bank
                                .frame(form, NpcAction::Walk, ((heading + 1) % 8) as u8, 1)
                                .unwrap()
                        }
                        3 => draw.actor.alive = false,
                        _ => {
                            draw.source_frame = 0;
                            draw.sample =
                                bank.frame(form, NpcAction::Walk, heading as u8, 0).unwrap();
                        }
                    }
                    let pixels = canvas.rgba().to_vec();
                    assert!(assets
                        .render_into_npcs(&scene, &mut canvas, &cancel, &[], &[], &[draw])
                        .is_err());
                    assert_eq!(canvas.rgba(), pixels);
                }
            }
        }
    }
    #[test]
    fn weapon_action_keys_preserve_old_unarmed_contract_and_class_qualification() {
        let base = BaseAppearance { class: 1, sex: 0 };
        for (ty, expected) in [
            (0, [3, 0, 1, 2, 8]),
            (1, [7, 4, 5, 6, 8]),
            (2, [49, 46, 47, 48, 8]),
            (3, [53, 50, 51, 52, 8]),
            (5, [27, 24, 25, 26, 8]),
            (6, [14, 11, 12, 13, 8]),
        ] {
            let armed = CombatAppearance {
                appearance: base,
                weapon_type: ty,
            };
            for (action, id) in [
                BaseAction::Idle,
                BaseAction::Walk,
                BaseAction::Attack,
                BaseAction::Damage,
                BaseAction::Death,
            ]
            .into_iter()
            .zip(expected)
            {
                assert_eq!(armed.action_key(action).unwrap(), ActionKey::new(id));
                if ty == 0 {
                    assert_eq!(armed.action_key(action).unwrap(), action.action_key());
                }
            }
        }
        assert!(CombatAppearance {
            appearance: base,
            weapon_type: 11
        }
        .action_key(BaseAction::Idle)
        .is_err());
        assert!(CombatAppearance {
            appearance: base,
            weapon_type: 18
        }
        .action_key(BaseAction::Death)
        .is_err());
    }

    #[test]
    fn same_base_appearance_different_weapons_use_distinct_cache_keys_and_missing_is_not_unarmed() {
        let mut assets = synthetic_assets();
        let appearance = BaseAppearance { class: 1, sex: 0 };
        let mut info = assets
            .animation_info(appearance, BaseAction::Idle, 0)
            .unwrap()
            .clone();
        info.frame_count = 1;
        info.total_ticks = 1;
        assets.animations.insert(
            (
                CombatAppearance {
                    appearance,
                    weapon_type: 1,
                },
                BaseAction::Idle,
                0,
            ),
            AnimationAssets {
                info,
                ends: vec![1],
                frames: vec![FrameBinding {
                    sprite: "3213-8.spr".into(),
                    subframe: 1,
                }],
            },
        );
        let (x, y) = key().world_origin();
        let a = actor(1, 7, x + 20, y + 20);
        let mut b = actor(2, 42, x + 21, y + 20);
        b.weapon_type = 1;
        let mut canvas = Canvas::new(800, 600).unwrap();
        let cancel = AtomicBool::new(false);
        let rendered = assets
            .render_into(
                &snapshot(assets.map(), 1, 1, vec![a.clone(), b.clone()]),
                &mut canvas,
                &cancel,
            )
            .unwrap();
        assert_eq!(
            rendered
                .actors
                .iter()
                .find(|p| p.entity_id == 1)
                .unwrap()
                .subframe,
            0
        );
        assert_eq!(
            rendered
                .actors
                .iter()
                .find(|p| p.entity_id == 2)
                .unwrap()
                .subframe,
            1
        );
        assert!(assets
            .animation_info_armed(appearance, 2, BaseAction::Idle, 0)
            .is_none());
        let saved = canvas.rgba().to_vec();
        b.weapon_type = 2;
        assert!(assets
            .render_into(
                &snapshot(assets.map(), 1, 2, vec![a, b]),
                &mut canvas,
                &cancel
            )
            .is_err());
        assert_eq!(canvas.rgba(), saved); // reject before touching previous surface
    }

    #[test]
    fn one_shot_override_replaces_body_and_preserves_true_resource_rectangle() {
        let assets = synthetic_assets();
        let (ox, oy) = key().world_origin();
        let scene = snapshot(assets.map(), 1, 1, vec![actor(1, 7, ox + 20, oy + 20)]);
        let mut canvas = Canvas::new(800, 600).unwrap();
        let cancel = AtomicBool::new(false);
        let original = assets.render_into(&scene, &mut canvas, &cancel).unwrap();
        assert_eq!(pixel(&canvas, 398, 296), [200, 0, 0, 255]);
        let frame = SpriteFrame {
            fragments: None,
            x_offset: 0,
            y_offset: 0,
            width: 2,
            height: 2,
            frame_type: 0,
            unknown1: 0,
            unknown2: 0,
            rgba: [0, 0, 250, 255].repeat(4),
        };
        let replacement = ActorOverride {
            entity_id: 1,
            frame: &frame,
            graphics_id: 999,
            source_frame: 2,
            sprite_name: "999-19.spr",
            subframe: 3,
        };
        let drawn = assets
            .render_into_overrides(&scene, &mut canvas, &cancel, &[replacement])
            .unwrap();
        assert_eq!(drawn.actors.len(), 1);
        assert_eq!(drawn.actors[0].anchor, original.actors[0].anchor);
        assert_eq!(drawn.actors[0].top_left, PixelPoint { x: 400, y: 300 });
        assert_eq!((drawn.actors[0].width, drawn.actors[0].height), (2, 2));
        assert_eq!(drawn.actors[0].sprite_name, "999-19.spr");
        assert_eq!(drawn.actors[0].graphics_id, 999);
        assert_eq!(
            (drawn.actors[0].source_frame, drawn.actors[0].subframe),
            (2, 3)
        );
        assert_ne!(
            pixel(&canvas, 398, 296),
            [200, 0, 0, 255],
            "old body must not remain under cast body"
        );
        assert_eq!(pixel(&canvas, 400, 300), [0, 0, 250, 255]);
        let saved = canvas.rgba().to_vec();
        let unknown = ActorOverride {
            entity_id: 2,
            frame: &frame,
            graphics_id: 999,
            source_frame: 2,
            sprite_name: "999-19.spr",
            subframe: 3,
        };
        assert!(assets
            .render_into_overrides(&scene, &mut canvas, &cancel, &[unknown])
            .is_err());
        assert_eq!(canvas.rgba(), saved);
    }

    #[test]
    fn camera_map_and_other_actor_move_from_authoritative_world_cells_without_io() {
        let assets = synthetic_assets();
        let (ox, oy) = key().world_origin();
        let a = actor(1001, 7, ox + 20, oy + 20);
        let b = actor(1002, 42, ox + 22, oy + 20);
        let first = snapshot(assets.map(), 1001, 1, vec![a.clone(), b.clone()]);
        let cancel = AtomicBool::new(false);
        let mut canvas = Canvas::new(800, 600).unwrap();
        let view = assets.render_into(&first, &mut canvas, &cancel).unwrap();
        assert_eq!(pixel(&canvas, 380, 290), [0, 200, 0, 255]);
        let center = view.actors.iter().find(|p| p.entity_id == 1001).unwrap();
        let other = view.actors.iter().find(|p| p.entity_id == 1002).unwrap();
        assert_eq!(center.anchor, PixelPoint { x: 400, y: 300 });
        assert_eq!(other.anchor, PixelPoint { x: 448, y: 276 });
        let a = ActorState {
            cell: WorldCell {
                x: a.cell.x + 1,
                y: a.cell.y,
            },
            heading: 2,
            action: BaseAction::Walk,
            action_elapsed: Duration::from_millis(100),
            ..a
        };
        let second = snapshot(assets.map(), 1001, 2, vec![b, a]);
        let moved = assets.render_into(&second, &mut canvas, &cancel).unwrap();
        assert_eq!(pixel(&canvas, 356, 302), [0, 200, 0, 255]);
        assert_eq!(pixel(&canvas, 380, 290), [18, 25, 35, 255]);
        let center = moved.actors.iter().find(|p| p.entity_id == 1001).unwrap();
        let other = moved.actors.iter().find(|p| p.entity_id == 1002).unwrap();
        assert_eq!(center.anchor, PixelPoint { x: 400, y: 300 });
        assert_eq!(other.anchor, PixelPoint { x: 424, y: 288 });
        assert_eq!(
            (center.graphics_id, center.source_frame, center.subframe),
            (3213, 1, 1)
        );
        assert_eq!(center.sprite_name, "3213-2.spr");
        assert_eq!(pixel(&canvas, 400, 298), [0, 2, 200, 255]);
        assert_eq!(
            moved.projection.pixel_to_cell(other.anchor),
            Some(other.cell)
        );
        assert_eq!(moved.revision, 2);
    }

    #[test]
    fn wrong_digest_unloaded_appearance_outside_position_and_cancel_do_not_publish() {
        let assets = synthetic_assets();
        let (ox, oy) = key().world_origin();
        let a = actor(1001, 7, ox + 20, oy + 20);
        let mut canvas = Canvas::new(80, 60).unwrap();
        canvas.clear([10, 20, 30]);
        let retained = canvas.rgba().to_vec();
        let wrong = SceneSnapshot::new(19, 1, [99; 32], 1001, vec![a.clone()]).unwrap();
        assert!(assets
            .render_into(&wrong, &mut canvas, &AtomicBool::new(false))
            .is_err());
        for bad in [
            ActorState {
                appearance: BaseAppearance { class: 0, sex: 0 },
                ..a.clone()
            },
            ActorState {
                cell: WorldCell {
                    x: i32::MAX,
                    y: i32::MIN,
                },
                ..a.clone()
            },
        ] {
            assert!(assets
                .render_into(
                    &snapshot(assets.map(), 1001, 1, vec![bad]),
                    &mut canvas,
                    &AtomicBool::new(false)
                )
                .is_err());
            assert_eq!(canvas.rgba(), retained);
        }
        assert!(assets
            .render_into(
                &snapshot(assets.map(), 1001, 1, vec![a]),
                &mut canvas,
                &AtomicBool::new(true)
            )
            .is_err());
        assert_eq!(canvas.rgba(), retained);
    }

    fn real_binding() -> SectorBinding {
        let raw = "ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271";
        let mut sha256 = [0; 32];
        for (index, byte) in sha256.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&raw[index * 2..index * 2 + 2], 16).unwrap();
        }
        SectorBinding {
            key: key(),
            byte_length: 137_784,
            sha256,
        }
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; actual shared map/effect queue, scope/target removal/restoration; optional owned PNGs"]
    fn real_disintegrate_layers_share_world_depth_and_survive_target_removal() {
        real_disintegrate_layers_share_world_depth_and_survive_target_removal_with_profile(crate::modern_lito::disintegrate_assets::DisintegrateAssets::load);
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; explicit current launcher TW profile; original CSharp SPR/pixel/clock assertions retained, no legacy root fallback"]
    fn real_disintegrate_layers_share_world_depth_and_survive_target_removal_current_launcher_tw_profile() {
        real_disintegrate_layers_share_world_depth_and_survive_target_removal_with_profile(crate::modern_lito::disintegrate_assets::DisintegrateAssets::load_current_tw);
    }

    fn real_disintegrate_layers_share_world_depth_and_survive_target_removal_with_profile(profile_loader: fn(&std::path::Path, &std::sync::atomic::AtomicBool) -> anyhow::Result<crate::modern_lito::disintegrate_assets::DisintegrateAssets>) {
        use super::super::{
            disintegrate_assets::DisintegrateAssets,
            disintegrate_effect::{Binding, DisintegrateEffect, Variant},
        };
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
        );
        let cancel = AtomicBool::new(false);
        let map = Arc::new(WorldMap::load_shared(&root, &[real_binding()], &cancel).unwrap());
        let scene_assets = WorldSceneAssets::load(
            &root,
            map.clone(),
            &[BaseAppearance { class: 1, sex: 0 }],
            SceneLimits::default(),
            &cancel,
        )
        .unwrap();
        let fx = Arc::new(profile_loader(&root, &cancel).unwrap());
        let selected = actor(1, 7, 32724, 32784);
        let target = actor(2, 42, 32725, 32784);
        let scene = snapshot(&map, 1, 1, vec![selected.clone(), target.clone()]);
        let binding = Binding {
            world_id: 19,
            map_digest: map.canonical_digest(),
            event_id: 1,
            start_revision: 1,
            target: target.cell,
        };
        let mut canvas = Canvas::new(800, 600).unwrap();
        let baseline_info = scene_assets
            .render_into(&scene, &mut canvas, &cancel)
            .unwrap();
        let baseline = canvas.rgba().to_vec();
        let capture = |label: &str, c: &Canvas| {
            if let Some(dir) = std::env::var_os("LITO_MODERN_DISINTEGRATE_CAPTURE_DIR") {
                let dir = std::path::PathBuf::from(dir);
                let target = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("target")
                    .canonicalize()
                    .unwrap();
                assert!(dir.is_absolute() && dir.canonicalize().unwrap().starts_with(&target));
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(dir.join(format!("disintegrate-{label}.png")))
                    .unwrap();
                let mut encoder = png::Encoder::new(file, 800, 600);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(c.rgba())
                    .unwrap();
            }
        };
        capture("baseline", &canvas);
        let mut hit_hashes = BTreeSet::new();
        for (variant, label) in [(Variant::Normal, "normal"), (Variant::Critical, "critical")] {
            let mut effect = DisintegrateEffect::start(fx.clone(), binding, variant).unwrap();
            for ms in [0, 167, 2375] {
                let elapsed = Duration::from_millis(ms);
                effect.advance(elapsed).unwrap();
                let draw = effect.draw(elapsed).unwrap().unwrap();
                let info = scene_assets
                    .render_into_npcs_roofs_effects(
                        &scene,
                        &mut canvas,
                        &cancel,
                        &[],
                        &[],
                        &[],
                        None,
                        &[draw],
                    )
                    .unwrap();
                assert_ne!(canvas.rgba(), baseline);
                // Screen/multiply never steal target ownership, including the
                // dark layer's covered black. Actual opaque walls still do.
                assert_eq!(info.actors.len(), baseline_info.actors.len());
                for (a, b) in info.actors.iter().zip(&baseline_info.actors) {
                    assert_eq!(a.visibility, b.visibility);
                }
                if ms == 167 {
                    hit_hashes.insert(format!("{:x}", Sha256::digest(canvas.rgba())));
                    capture(label, &canvas);
                }
            }
            assert!(
                effect
                    .advance(Duration::from_millis(2500))
                    .unwrap()
                    .finished
            );
            assert!(effect.draw(Duration::from_millis(2500)).unwrap().is_none());
            scene_assets
                .render_into_npcs_roofs_effects(
                    &scene,
                    &mut canvas,
                    &cancel,
                    &[],
                    &[],
                    &[],
                    None,
                    &[],
                )
                .unwrap();
            assert_eq!(canvas.rgba(), baseline);
        }
        assert_eq!(hit_hashes.len(), 2);
        capture("finished", &canvas);
        let effect = DisintegrateEffect::start(fx.clone(), binding, Variant::Normal).unwrap();
        let ms = Duration::from_millis(167);
        let removed = snapshot(&map, 1, 2, vec![selected.clone()]);
        scene_assets
            .render_into(&removed, &mut canvas, &cancel)
            .unwrap();
        let no_target = canvas.rgba().to_vec();
        scene_assets
            .render_into_npcs_roofs_effects(
                &removed,
                &mut canvas,
                &cancel,
                &[],
                &[],
                &[],
                None,
                &[effect.draw(ms).unwrap().unwrap()],
            )
            .unwrap();
        assert_ne!(canvas.rgba(), no_target);
        capture("target-removed", &canvas);
        assert_eq!(effect.binding().target, target.cell);
        let moved = super::super::world_map::DIRECTIONS
            .into_iter()
            .map(|(dx, dy)| WorldCell {
                x: target.cell.x + dx,
                y: target.cell.y + dy,
            })
            .find(|p| *p != selected.cell && map.can_step(target.cell, *p))
            .unwrap();
        let mut moved_target = target.clone();
        moved_target.cell = moved;
        let moved_scene = snapshot(&map, 1, 3, vec![selected.clone(), moved_target]);
        scene_assets
            .render_into_npcs_roofs_effects(
                &moved_scene,
                &mut canvas,
                &cancel,
                &[],
                &[],
                &[],
                None,
                &[effect.draw(ms).unwrap().unwrap()],
            )
            .unwrap();
        assert_eq!(
            effect.draw(ms).unwrap().unwrap().binding().target,
            target.cell
        );
        capture("target-moved", &canvas);
        // Moving the camera projects the same captured cell, without disk IO or
        // dependence on the target actor still being present.
        let camera_scene = snapshot(&map, 1, 4, vec![actor(1, 7, moved.x, moved.y)]);
        scene_assets
            .render_into_npcs_roofs_effects(
                &camera_scene,
                &mut canvas,
                &cancel,
                &[],
                &[],
                &[],
                None,
                &[effect.draw(ms).unwrap().unwrap()],
            )
            .unwrap();
        let preserved = canvas.rgba().to_vec();
        for bad in [
            Binding {
                world_id: 20,
                ..binding
            },
            Binding {
                map_digest: [7; 32],
                ..binding
            },
            Binding {
                start_revision: 2,
                ..binding
            },
            Binding {
                target: WorldCell { x: 1, y: 1 },
                ..binding
            },
        ] {
            let bad_effect = DisintegrateEffect::start(fx.clone(), bad, Variant::Normal).unwrap();
            assert!(scene_assets
                .render_into_npcs_roofs_effects(
                    &scene,
                    &mut canvas,
                    &cancel,
                    &[],
                    &[],
                    &[],
                    None,
                    &[bad_effect.draw(ms).unwrap().unwrap()]
                )
                .is_err());
            assert_eq!(canvas.rgba(), preserved);
        }
        let duplicate = [
            effect.draw(ms).unwrap().unwrap(),
            effect.draw(ms).unwrap().unwrap(),
        ];
        assert!(scene_assets
            .render_into_npcs_roofs_effects(
                &scene,
                &mut canvas,
                &cancel,
                &[],
                &[],
                &[],
                None,
                &duplicate
            )
            .is_err());
        let too_many: Vec<_> = (0..65).map(|_| effect.draw(ms).unwrap().unwrap()).collect();
        assert!(scene_assets
            .render_into_npcs_roofs_effects(
                &scene,
                &mut canvas,
                &cancel,
                &[],
                &[],
                &[],
                None,
                &too_many
            )
            .is_err());
        assert!(scene_assets
            .render_into_npcs_roofs_effects(
                &scene,
                &mut canvas,
                &AtomicBool::new(true),
                &[],
                &[],
                &[],
                None,
                &[effect.draw(ms).unwrap().unwrap()]
            )
            .is_err());
        assert_eq!(canvas.rgba(), preserved);
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; all 144 armed families in class-sized bounded batches, true SPR/RGBA and map render; no output"]
    fn real_shared_armed_families_preload_and_knight_pixels_match_independent_csharp_goldens() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root");
        let root = Path::new(&root);
        let cancel = AtomicBool::new(false);
        let map = Arc::new(WorldMap::load_shared(root, &[real_binding()], &cancel).unwrap());
        let mut families = 0;
        for class in 0..7 {
            let appearances: Vec<_> = (0..2)
                .flat_map(|sex| {
                    (0..=18).map(move |weapon_type| CombatAppearance {
                        appearance: BaseAppearance { class, sex },
                        weapon_type,
                    })
                })
                .filter(|a| qualify_weapon_appearance(a.appearance, a.weapon_type).is_ok())
                .collect();
            let assets = WorldSceneAssets::load_combat_armed(
                root,
                map.clone(),
                &appearances,
                SceneLimits {
                    max_encoded_bytes: 128 * 1024 * 1024,
                    max_texture_bytes: 256 * 1024 * 1024,
                    ..Default::default()
                },
                &cancel,
            )
            .unwrap();
            families += appearances.len();
            assert_eq!(assets.combat_animations().count(), appearances.len() * 3);
            for armed in &appearances {
                let walk = &assets.walks[armed];
                for heading in 0..8 {
                    let animation = &assets.animations[&(*armed, BaseAction::Walk, heading)];
                    for source in 0..walk.source_frames().len() {
                        let expected = walk.frame_asset(source, heading).unwrap();
                        assert_eq!(animation.frames[source].sprite, expected.file_name());
                        assert_eq!(
                            animation.frames[source].subframe,
                            expected.subframe as usize
                        );
                    }
                }
                if class == 1 && armed.appearance.sex == 0 && [1, 4].contains(&armed.weapon_type) {
                    verify_real_player_walk_placement(&assets, &map, *armed, &cancel);
                }
                for action in [BaseAction::Attack, BaseAction::Damage, BaseAction::Death] {
                    let qualified = assets
                        .combat_animation_armed(armed.appearance, armed.weapon_type, action)
                        .unwrap();
                    assert_eq!(qualified.provenance().weapon_type, armed.weapon_type);
                    assert_eq!(
                        qualified.provenance().action_key,
                        armed.action_key(action).unwrap()
                    );
                    for heading in 0..8 {
                        let animation = &assets.animations[&(*armed, action, heading)];
                        for elapsed in [Duration::ZERO, Duration::from_millis(250), Duration::MAX] {
                            let expected = qualified.start().advance(elapsed, heading).unwrap();
                            let index = animation.source_frame(elapsed);
                            assert_eq!(index, expected.source_frame);
                            assert_eq!(animation.frames[index].sprite, expected.asset.file_name());
                            assert_eq!(
                                animation.frames[index].subframe,
                                expected.asset.subframe as usize
                            );
                        }
                    }
                }
            }
            if class == 1 {
                // These SHA256s come from independent PakViewer SprReader.LoadRaw
                // on the exact hash-routed shared entries, not this Rust decoder.
                for (name, raw, rgba, size, w, h) in [
                    (
                        "3213-184.spr",
                        "99b4a14df99b70d492ac65a69f3caebf3ee11f8a366585e7577ae77e6b256b56",
                        "1c0cd7b1132b7202160a68a9642ae655c4c39e8feb4a658c2382169ee458f839",
                        10447,
                        72,
                        96,
                    ),
                    (
                        "3213-152.spr",
                        "0808fdf1254290fff55b124e70475620f214e8f4fc9e4c0fc12009c96beb9db6",
                        "4f6065087a469d677e59ed0821f39a55b62783fb4cf548a26a937a7fc07d0575",
                        7250,
                        48,
                        84,
                    ),
                    (
                        "3213-120.spr",
                        "6e2e0af223198c530bcfd0ba5c0ea2e3580ecb10ef7605d7e03666f8b5cb721b",
                        "729a3aa342148c13f9483867f8d78b711251cd710e2289a5db289daee503afff",
                        12581,
                        96,
                        84,
                    ),
                ] {
                    let sprite = &assets.sprites[name];
                    let frame = &sprite.decoded.frames[0];
                    assert_eq!(sprite.source.archive, SharedArchiveKind::Sprite);
                    assert_eq!(sprite.source.encoded_bytes, size);
                    assert_eq!(
                        sprite
                            .source
                            .sha256
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>(),
                        raw
                    );
                    assert_eq!(
                        (frame.x_offset, frame.y_offset, frame.width, frame.height),
                        (0, -48, w, h)
                    );
                    assert_eq!(format!("{:x}", Sha256::digest(&frame.rgba)), rgba);
                }
                let mut a = actor(1, 7, 32724, 32784);
                a.weapon_type = 1;
                a.action = BaseAction::Attack;
                let mut b = actor(2, 42, 32725, 32784);
                b.weapon_type = 2;
                b.action = BaseAction::Attack;
                let mut canvas = Canvas::new(800, 600).unwrap();
                let info = assets
                    .render_into(&snapshot(&map, 1, 1, vec![a, b]), &mut canvas, &cancel)
                    .unwrap();
                assert_eq!(
                    info.actors
                        .iter()
                        .find(|p| p.entity_id == 1)
                        .unwrap()
                        .sprite_name,
                    "3213-184.spr"
                );
                assert_eq!(
                    info.actors
                        .iter()
                        .find(|p| p.entity_id == 2)
                        .unwrap()
                        .sprite_name,
                    "3213-152.spr"
                );
            }
        }
        assert_eq!(families, 144);
    }

    fn verify_real_player_walk_placement(
        assets: &WorldSceneAssets,
        map: &WorldMap,
        armed: CombatAppearance,
        cancel: &AtomicBool,
    ) {
        let (x, y) = real_binding().key.world_origin();
        let from = (0..64)
            .flat_map(|dx| {
                (0..64).map(move |dy| WorldCell {
                    x: x + dx,
                    y: y + dy,
                })
            })
            .find(|p| {
                super::super::world_map::DIRECTIONS.iter().all(|(dx, dy)| {
                    map.can_step(
                        *p,
                        WorldCell {
                            x: p.x + dx,
                            y: p.y + dy,
                        },
                    )
                })
            })
            .expect("real map has an open eight-way neighborhood");
        let mut canvas = Canvas::new(800, 600).unwrap();
        let walk = &assets.walks[&armed];
        for (heading, (dx, dy)) in super::super::world_map::DIRECTIONS.into_iter().enumerate() {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            let mut a = actor(1001, 7, to.x, to.y);
            a.appearance = armed.appearance;
            a.weapon_type = armed.weapon_type;
            a.action = BaseAction::Walk;
            a.heading = heading as u8;
            let scene = snapshot(map, 1001, 9, vec![a.clone()]);
            for elapsed in [
                Duration::ZERO,
                walk.active_duration() / 2,
                walk.active_duration(),
            ] {
                // Identity pacing isolates native source boundaries from the
                // independently tested modern authority-to-animation mapping.
                let placement = assets
                    .player_walk(
                        &scene,
                        1001,
                        from,
                        elapsed,
                        walk.step_duration().as_nanos() as u64,
                        cancel,
                    )
                    .unwrap();
                let expected_source = if elapsed.is_zero() {
                    1
                } else if elapsed == walk.active_duration() {
                    0
                } else {
                    placement.source
                };
                let terminal = elapsed == walk.active_duration();
                let expected_anchor = if terminal { to } else { from };
                let info = assets
                    .render_into_npcs_roofs_effects_walking(
                        &scene,
                        &mut canvas,
                        cancel,
                        &[],
                        &[],
                        &[],
                        None,
                        &[],
                        &[placement],
                    )
                    .unwrap();
                assert_eq!(info.revision, 9);
                assert_eq!(info.actors[0].cell, to);
                assert_eq!(info.actors[0].source_frame, expected_source);
                assert_eq!(info.projection.anchor_cell, expected_anchor);
                assert_eq!(
                    info.actors[0].anchor,
                    info.projection.cell_to_pixel(expected_anchor).unwrap()
                );
                let expected = walk.frame_asset(expected_source, heading as u8).unwrap();
                assert_eq!(info.actors[0].sprite_name, expected.file_name());
                assert_eq!(info.actors[0].subframe, expected.subframe as usize);
                if elapsed.is_zero() {
                    let (px, py) = walk.local_camera_pulse(0, heading as u8).unwrap();
                    assert_eq!(
                        info.projection.anchor_pixel,
                        PixelPoint {
                            x: 400 - px,
                            y: 300 - py
                        }
                    );
                } else if terminal {
                    assert_eq!(info.projection.anchor_pixel, PixelPoint { x: 400, y: 300 });
                }
                let retained = canvas.rgba().to_vec();
                let stale = assets
                    .player_walk(
                        &scene,
                        1001,
                        from,
                        elapsed,
                        walk.step_duration().as_nanos() as u64,
                        cancel,
                    )
                    .unwrap();
                assert!(assets
                    .render_into_npcs_roofs_effects_walking(
                        &snapshot(map, 1001, 10, vec![a.clone()]),
                        &mut canvas,
                        cancel,
                        &[],
                        &[],
                        &[],
                        None,
                        &[],
                        &[stale],
                    )
                    .is_err());
                assert_eq!(canvas.rgba(), retained);
            }
        }
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; all shared unarmed combat art plus real world514 render; no output/playback"]
    fn real_shared_combat_assets_match_qualified_one_shots_and_render_attack_hurt_death() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root");
        let cancel = AtomicBool::new(false);
        let map =
            Arc::new(WorldMap::load_shared(Path::new(&root), &[real_binding()], &cancel).unwrap());
        let appearances: Vec<_> = (0..7)
            .flat_map(|class| (0..2).map(move |sex| BaseAppearance { class, sex }))
            .collect();
        let assets = WorldSceneAssets::load_combat(
            Path::new(&root),
            map.clone(),
            &appearances,
            SceneLimits::default(),
            &cancel,
        )
        .unwrap();
        assert_eq!(assets.combat.len(), 42);
        for appearance in appearances {
            for action in [BaseAction::Attack, BaseAction::Damage, BaseAction::Death] {
                let qualified = assets.combat_animation(appearance, action).unwrap();
                for heading in 0..8 {
                    let animation = &assets.animations[&(
                        CombatAppearance {
                            appearance,
                            weapon_type: 0,
                        },
                        action,
                        heading,
                    )];
                    for elapsed in [Duration::ZERO, Duration::from_millis(200), Duration::MAX] {
                        let expected = qualified.start().advance(elapsed, heading).unwrap();
                        let actual = animation.source_frame(elapsed);
                        assert_eq!(actual, expected.source_frame);
                        assert_eq!(animation.frames[actual].sprite, expected.asset.file_name());
                        assert_eq!(
                            animation.frames[actual].subframe,
                            expected.asset.subframe as usize
                        );
                    }
                }
            }
        }
        let mut canvas = Canvas::new(800, 600).unwrap();
        let mut hashes = BTreeSet::new();
        for (revision, action) in [BaseAction::Attack, BaseAction::Damage, BaseAction::Death]
            .into_iter()
            .enumerate()
        {
            let mut a = actor(1001, 7, 32724, 32784);
            a.action = action;
            let mut b = actor(1002, 42, 32725, 32784);
            b.appearance = BaseAppearance { class: 3, sex: 1 };
            b.action = BaseAction::Damage;
            let info = assets
                .render_into(
                    &snapshot(&map, 1001, revision as u64 + 1, vec![a.clone(), b.clone()]),
                    &mut canvas,
                    &cancel,
                )
                .unwrap();
            assert!(info.actors.iter().all(|a| a.source_frame >= 1));
            hashes.insert(format!("{:x}", Sha256::digest(canvas.rgba())));
            a.action_elapsed = Duration::MAX;
            b.action_elapsed = Duration::MAX;
            let end = assets
                .render_into(
                    &snapshot(&map, 1001, revision as u64 + 4, vec![a, b]),
                    &mut canvas,
                    &cancel,
                )
                .unwrap();
            assert!(end.actors.iter().all(|a| a.source_frame == 0));
        }
        assert_eq!(hashes.len(), 3);
        assert!(
            assets
                .limitations
                .weapon_and_transformation_art_unimplemented
        );
        eprintln!(
            "shared combat render: 42 one-shots, {} SPR entries, {} encoded / {} decoded bytes; no cue playback",
            assets.sprite_sources().count(),
            assets.encoded_bytes(),
            assets.texture_bytes()
        );
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; all 14 base appearances/8 headings plus world514 movement, no output files"]
    fn real_shared_world_all_base_appearances_decode_and_authoritative_camera_moves() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root");
        let cancel = AtomicBool::new(false);
        let map =
            Arc::new(WorldMap::load_shared(Path::new(&root), &[real_binding()], &cancel).unwrap());
        let appearances: Vec<_> = (0..7)
            .flat_map(|class| (0..2).map(move |sex| BaseAppearance { class, sex }))
            .collect();
        let assets = WorldSceneAssets::load(
            Path::new(&root),
            map.clone(),
            &appearances,
            SceneLimits::default(),
            &cancel,
        )
        .unwrap();
        let expected = [
            [3225, 3227],
            [3213, 3215],
            [3217, 3474],
            [3476, 3221],
            [2786, 2796],
            [6658, 6661],
            [6671, 6650],
        ];
        for appearance in &appearances {
            for action in [BaseAction::Idle, BaseAction::Walk] {
                for heading in 0..8 {
                    let info = assets.animation_info(*appearance, action, heading).unwrap();
                    assert_eq!(
                        info.logical_sprite_id,
                        appearance.logical_sprite_id().unwrap()
                    );
                    assert_eq!(
                        info.graphics_id,
                        expected[appearance.class as usize][appearance.sex as usize]
                    );
                    assert_eq!(info.heading, heading);
                    assert!(info.frame_count > 0 && info.total_ticks > 0);
                }
            }
        }
        assert_eq!(assets.sprite_sources().count(), 224);
        assert_eq!(assets.tile_set_count(), 56);
        // Independent C# 1.5.4 decoded all 224 native-routed entries. Actor
        // RGBA remains 23,609,088. C# S32Reader independently reports 5,064
        // distinct Layer1/2/4 references. Only those map tiles are retained;
        // encoded archives and the existing rendered pixel goldens stay intact.
        let required: BTreeSet<_> = assets
            .draws
            .iter()
            .map(|d| (d.tile.archive_entry_id, d.tile.sub_index as usize))
            .collect();
        let retained: BTreeSet<_> = assets
            .tiles
            .iter()
            .flat_map(|(id, tiles)| tiles.keys().map(|sub| (*id, *sub)))
            .collect();
        assert_eq!(retained, required);
        assert_eq!(retained.len(), 5_064);
        assert_eq!(assets.encoded_bytes(), 2_461_731 + 5_871_648);
        let flattened: usize = assets
            .sprites
            .values()
            .flat_map(|s| &s.decoded.frames)
            .map(|f| f.rgba.len())
            .sum();
        assert_eq!(flattened, 23_609_088);
        assert!(assets
            .sprites
            .values()
            .flat_map(|s| &s.decoded.frames)
            .all(|f| f.fragments.is_some()));
        let retained: usize = assets
            .sprites
            .values()
            .map(|s| s.decoded.retained_rgba_bytes().unwrap())
            .sum();
        let atlas_bytes = retained - flattened;
        // Independent raw SPR byte count bounds the additional compact data;
        // retaining a 24x24 RGBA atlas inflated this by 36,345,600 bytes.
        assert!(
            atlas_bytes > 0 && atlas_bytes < 2 * 2_461_731,
            "compact atlas bytes {atlas_bytes}"
        );
        assert_eq!(super::super::compositor::FragmentRows::BYTES, 72);
        assert_eq!(
            assets.texture_bytes(),
            retained + 5_064 * (24 * 24 * 4 + 72)
        );
        eprintln!(
            "native world retained: flat={flattened}, compact={atlas_bytes}, total={}",
            assets.texture_bytes()
        );
        for (name, raw_sha, frame_count, rgba_sha) in [
            (
                "3213-3.spr",
                "BCBBB70F9A9559D59AFFB79688CE66B7B2D06CF10D1F695FE3203A77C6FC67F4",
                4,
                "2FD87BD98E5DA6A5CBB9DDDFFDD6482A93767FBAD0CCA63198742466B740BD62",
            ),
            (
                "3213-8.spr",
                "34FD08BCF35442ED10968B079E3DBA7B81AC4CC61991AABCFF6498786E502FD7",
                13,
                "0E257AB768948880B2B04D0170EA21E7787EF45C19CD7294D370818ACE0407EE",
            ),
            (
                "3221-8.spr",
                "495B667BEF143F93251CC18CFD053822A1E64418C324CBC30EEAD9226035F61B",
                8,
                "C59E961E7649B75E36564732D0893D5B315AC098245F6AA79E13B74CB490B488",
            ),
        ] {
            let sprite = &assets.sprites[name];
            assert_eq!(sprite.source.archive, SharedArchiveKind::Sprite);
            let actual_sha: String = sprite
                .source
                .sha256
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect();
            assert_eq!(actual_sha, raw_sha);
            assert_eq!(sprite.decoded.frames.len(), frame_count);
            let frame = &sprite.decoded.frames[0];
            assert_eq!(
                (frame.x_offset, frame.y_offset, frame.width, frame.height),
                (0, -48, 48, 72)
            );
            assert_eq!(format!("{:X}", Sha256::digest(&frame.rgba)), rgba_sha);
        }
        assert_eq!(assets.limitations().opacity_records, 17);
        assert_eq!(assets.limitations().sprite_effects, 36);
        assert!(!assets.limitations().actor_object_occlusion_unimplemented);
        assert!(!assets.limitations().native_actor_foot_pivot_unverified);
        assert!(!assets.limitations().actor_dependent_roofs_unimplemented);
        assert!(
            assets
                .limitations()
                .native_roof_timers_and_aliases_unverified
        );
        assert!(assets.limitations().native_actor_update_order_unverified);
        assert!(
            assets
                .limitations()
                .weapon_and_transformation_art_unimplemented
        );
        assert!(assets.limitations().unplayed_sound_ids.is_empty());
        assert_eq!(assets.limitations().unrendered_base_effect_ids, [478, 479]);
        assert_eq!(assets.limitations().unrendered_base_effects.len(), 28);
        for appearance in &appearances {
            let cues: Vec<_> = assets
                .limitations()
                .unrendered_base_effects
                .iter()
                .filter(|cue| cue.appearance == *appearance)
                .collect();
            assert_eq!(cues.len(), 2);
            assert!(cues.iter().all(|cue| cue.action == BaseAction::Walk));
            assert!(cues
                .iter()
                .all(|cue| cue.tick == (cue.frame_index as u64) * 4));
            assert_eq!(
                cues.iter()
                    .map(|cue| cue.effect_id)
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([478, 479])
            );
        }
        let a = actor(1001, 7, 32724, 32784);
        let b = ActorState {
            appearance: BaseAppearance { class: 3, sex: 1 },
            ..actor(1002, 42, 32725, 32784)
        };
        let destination = WorldCell { x: 32725, y: 32785 };
        assert!(map.can_step(a.cell, destination));
        map.validate_spawns(&[a.cell, b.cell, destination], &cancel)
            .unwrap();
        let mut canvas = Canvas::new(800, 600).unwrap();
        let first = assets
            .render_into(
                &snapshot(&map, 1001, 1, vec![a.clone(), b.clone()]),
                &mut canvas,
                &cancel,
            )
            .unwrap();
        let first_hash = format!("{:x}", Sha256::digest(canvas.rgba()));
        // Independent C# Tile/SPR pixels + literal per-cell insertion oracle,
        // tools/verify-world-fragment-reference.ps1. Shared action3 starts at
        // 3213-8:12 and 3221-8:7, not the first physical SPR subframes.
        assert_eq!(
            first_hash,
            "9c5b493f6f09d276128f0d38b7f85c1c34d42f82f7eb0287bebb7e538e69ce6c"
        );
        assert_eq!(
            first.actors.iter().map(|p| p.entity_id).collect::<Vec<_>>(),
            [1002, 1001]
        );
        let first_a = first.actors.iter().find(|p| p.entity_id == 1001).unwrap();
        let first_b = first.actors.iter().find(|p| p.entity_id == 1002).unwrap();
        assert_eq!([first_a.subframe, first_b.subframe], [12, 7]);
        assert!(first.actors.iter().all(|p| p.visibility.is_some()));
        let mut owners = Sha256::new();
        for y in 0..600 {
            for x in 0..800 {
                let a = first_a.visibility.as_ref().unwrap().contains(x, y);
                let b = first_b.visibility.as_ref().unwrap().contains(x, y);
                assert!(!(a && b), "one painted pixel cannot belong to both actors");
                owners.update(
                    (if a {
                        1u16
                    } else if b {
                        2u16
                    } else {
                        0u16
                    })
                    .to_le_bytes(),
                );
            }
        }
        assert_eq!(
            format!("{:x}", owners.finalize()),
            "c1a75f153c4040db08cb3ea32d3732838e95798d90dda44a334e641153f92de1"
        );
        let other_before = first
            .actors
            .iter()
            .find(|actor| actor.entity_id == 1002)
            .unwrap()
            .anchor;
        let moved = ActorState {
            cell: destination,
            heading: 3,
            action: BaseAction::Walk,
            action_elapsed: Duration::from_millis(200),
            ..a
        };
        let second = assets
            .render_into(
                &snapshot(&map, 1001, 2, vec![moved, b]),
                &mut canvas,
                &cancel,
            )
            .unwrap();
        let second_hash = format!("{:x}", Sha256::digest(canvas.rgba()));
        assert_ne!(first_hash, second_hash);
        let selected = second
            .actors
            .iter()
            .find(|actor| actor.entity_id == 1001)
            .unwrap();
        let other = second
            .actors
            .iter()
            .find(|actor| actor.entity_id == 1002)
            .unwrap();
        assert_eq!(selected.graphics_id, 3213);
        assert_eq!(selected.sprite_name, "3213-3.spr");
        assert_eq!(selected.anchor, PixelPoint { x: 400, y: 300 });
        assert_eq!(other.graphics_id, 3221);
        assert_eq!(
            other.anchor,
            PixelPoint {
                x: other_before.x - 48,
                y: other_before.y
            }
        );
        assert_eq!(
            second.projection.pixel_to_cell(other.anchor),
            Some(other.cell)
        );
        assert!(second.visible_tiles > 0);
        if std::env::var("LITO_WORLD_SCENE_WRITE_REFERENCE").as_deref() == Ok("1") {
            let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target/modern-lito-render-tests/shared-map514-actors-reference.png");
            std::fs::create_dir_all(output.parent().unwrap()).unwrap();
            let file = std::fs::File::create(&output).unwrap();
            let mut encoder = png::Encoder::new(file, 800, 600);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(canvas.rgba())
                .unwrap();
            eprintln!("static actor reference: {}", output.display());
        }
        eprintln!(
            "world_scene appearances14/directions8/idle+walk loaded; sprites={} tilesets={} encoded={} texture={} firstRGBA={} movedRGBA={}; native fragments/anchor and roof kind0/1 integrated; effects36/roof-timer-alias/native-update-order/unarmed-only limitations retained",
            assets.sprite_sources().count(),
            assets.tile_set_count(),
            assets.encoded_bytes(),
            assets.texture_bytes(),
            first_hash,
            second_hash
        );
    }
}
