//! Authority-confirmed world entry and movement. No client packet grants a lease,
//! and PREPARING is not a loaded/active world. Wire integers are big endian.
use super::application_protocol::{self as app, ApplicationError, CharacterKey, SelectionContext};
pub use super::participant::{MemberKind, ParticipantId};
use super::{rooms, world_map::WorldCell};

pub const REQUEST_KIND: u16 = 20;
pub const RESULT_KIND: u16 = 21;
pub const REQUEST_BYTES: usize = 104;
pub const MAX_FRAME_BYTES: usize = 128 * 1024;
/// Nested room/world frames share the implemented application ceiling. Optional
/// v32/v33 chat does not add bytes to the v31 world/combat payloads.
pub(crate) const CODEC_MAX_VERSION: u16 = super::protocol::IMPLEMENTED_WORLD_VERSION;

/// A match policy is not a source/character permission or a new native ruleset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MatchRuleset {
    SceneOnly = 0,
    PvpSkirmish = 1,
    DeathmatchElimination = 2,
    HonorTowers = 3,
    PetRace = 4,
    NpcEncounter = 5,
    CooperativeTimeAttack = 6,
    SiegeObjectives = 7,
}

/// v15 schema 1. These are actual immutable server limits, not a countdown
/// receipt. Clock kind is server monotonic. Dedicated mode appendices decide
/// whether expiry is a result; legacy generic expiry remains an abort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchDescriptor {
    pub ruleset: MatchRuleset,
    pub features: u32,
    pub preparation_timeout_millis: u32,
    pub load_timeout_millis: u32,
    pub countdown_nanos: u64,
    pub active_timeout_millis: u32,
}
impl MatchDescriptor {
    pub const COMBAT: u32 = 1;
    pub const SPELLS: u32 = 2;
    pub const LIGHT_ARROW: u32 = 4;
    pub const ITEMS: u32 = 8;
    pub const WEIGHT: u32 = 16;
    pub const TIMED: u32 = 32;
    pub const AMMO: u32 = 64;
    pub const REGEN: u32 = 128;
    pub const HONOR: u32 = 256;
    pub const FIRE_STORM: u32 = 512;
    pub const PET_RACE: u32 = 1024;
    pub const NPC: u32 = 2048;
    pub const ISOLATED_FORCES: u32 = 4096;
    pub const QUEST_OBJECTIVES: u32 = 8192;
    pub const QUEST_LIVES: u32 = 16384;
    pub const NPC_LIGHT_ARROW: u32 = 32768;
    pub const TIME_ATTACK: u32 = 65536;
    pub const SIEGE: u32 = 131072;
    pub const DISINTEGRATE: u32 = 262144;
    pub const NPC_DISINTEGRATE: u32 = 524288;
    pub const NPC_FIRE_STORM: u32 = 1048576;
    pub const KNOWN_FEATURES: u32 = 2097151;
    pub const GREATER_HASTE: u32 = 2097152;
    pub(crate) const HASTE_CANDIDATE: u32 = 4194304;
    pub(crate) const SLOW_CANDIDATE: u32 = 8388608;
    pub(crate) const PLAYER_HEALING_CANDIDATE: u32 = 16777216;
    pub(crate) const ARMOR_PROTECTION_CANDIDATE: u32 = 33554432;
    pub(crate) const FULL_VIP_CANDIDATE: u32 = 67108864;
    pub const WIRE_BYTES: usize = 32;

    pub fn has(&self, feature: u32) -> bool {
        self.features & feature == feature
    }
    pub fn combat(&self) -> bool {
        self.has(Self::COMBAT)
    }
    pub fn results(&self) -> bool {
        matches!(
            self.ruleset,
            MatchRuleset::DeathmatchElimination | MatchRuleset::HonorTowers
        ) || self.has(Self::QUEST_OBJECTIVES)
    }
    pub fn validate(&self, mode: u8) -> Result<(), ApplicationError> {
        self.validate_features(mode, Self::KNOWN_FEATURES)
    }
    pub(crate) fn validate_canonical(&self, mode: u8) -> Result<(), ApplicationError> {
        self.validate_features(mode, Self::KNOWN_FEATURES | Self::GREATER_HASTE)
    }
    /// Candidate data only; existing negotiation and codecs still reject it.
    pub(crate) fn validate_haste_candidate(&self, mode: u8) -> Result<(), ApplicationError> {
        if !self.has(Self::HASTE_CANDIDATE) {
            return Err(invalid());
        }
        self.validate_features(
            mode,
            Self::KNOWN_FEATURES | Self::GREATER_HASTE | Self::HASTE_CANDIDATE,
        )
    }
    /// Schema6 candidate only: this does not enable negotiated versions.
    pub(crate) fn validate_speed_candidate(&self, mode: u8) -> Result<(), ApplicationError> {
        if !self.has(Self::SLOW_CANDIDATE) {
            return Err(invalid());
        }
        self.validate_features(
            mode,
            Self::KNOWN_FEATURES
                | Self::GREATER_HASTE
                | Self::HASTE_CANDIDATE
                | Self::SLOW_CANDIDATE,
        )
    }
    pub(crate) fn validate_healing_candidate(&self, mode: u8) -> Result<(), ApplicationError> {
        if !self.has(Self::PLAYER_HEALING_CANDIDATE) {
            return Err(invalid());
        }
        self.validate_features(
            mode,
            Self::KNOWN_FEATURES
                | Self::GREATER_HASTE
                | Self::HASTE_CANDIDATE
                | Self::SLOW_CANDIDATE
                | Self::PLAYER_HEALING_CANDIDATE,
        )
    }
    /// Internal schema8 proof only. Live capability/descriptor encoders keep
    /// their existing masks and cannot admit armor from a version number.
    pub(crate) fn validate_armor_candidate(&self, mode: u8) -> Result<(), ApplicationError> {
        if !self.has(Self::ARMOR_PROTECTION_CANDIDATE) {
            return Err(invalid());
        }
        self.validate_features(
            mode,
            Self::KNOWN_FEATURES
                | Self::GREATER_HASTE
                | Self::HASTE_CANDIDATE
                | Self::SLOW_CANDIDATE
                | Self::PLAYER_HEALING_CANDIDATE
                | Self::ARMOR_PROTECTION_CANDIDATE,
        )
    }
    /// Complete schema10 candidate only. Never added to a legacy/live mask.
    pub(crate) fn validate_vip_candidate(&self, mode: u8) -> Result<(), ApplicationError> {
        let known = Self::FULL_VIP_CANDIDATE
            | 255
            | Self::FIRE_STORM
            | Self::DISINTEGRATE
            | Self::GREATER_HASTE
            | Self::HASTE_CANDIDATE
            | Self::SLOW_CANDIDATE
            | Self::PLAYER_HEALING_CANDIDATE
            | Self::ARMOR_PROTECTION_CANDIDATE;
        if mode != 1
            || !self.has(Self::FULL_VIP_CANDIDATE | Self::COMBAT)
            || !matches!(
                self.ruleset,
                MatchRuleset::PvpSkirmish | MatchRuleset::DeathmatchElimination
            )
        {
            return Err(invalid());
        }
        self.validate_features(mode, known)
    }
    fn validate_features(&self, mode: u8, known: u32) -> Result<(), ApplicationError> {
        if mode > 5
            || self.features & !known != 0
            || !(50..=30_000).contains(&self.preparation_timeout_millis)
            || !(50..=30_000).contains(&self.load_timeout_millis)
            || self.countdown_nanos > 30_000_000_000
            || !(50..=1_800_000).contains(&self.active_timeout_millis)
        {
            return Err(invalid());
        }
        for (feature, dependencies) in [
            (Self::SPELLS, Self::COMBAT),
            (Self::LIGHT_ARROW, Self::SPELLS),
            (Self::ITEMS, Self::COMBAT),
            (Self::WEIGHT, Self::ITEMS),
            (Self::TIMED, Self::WEIGHT),
            (Self::AMMO, Self::TIMED),
            (Self::REGEN, Self::WEIGHT),
            (Self::HONOR, Self::REGEN | Self::LIGHT_ARROW),
            (Self::FIRE_STORM, Self::REGEN | Self::LIGHT_ARROW),
            (Self::ISOLATED_FORCES, Self::NPC),
            (Self::QUEST_OBJECTIVES, Self::NPC | Self::ISOLATED_FORCES),
            (Self::QUEST_LIVES, Self::QUEST_OBJECTIVES | Self::REGEN),
            (Self::NPC_LIGHT_ARROW, Self::NPC | Self::SPELLS),
            (Self::DISINTEGRATE, Self::REGEN | Self::LIGHT_ARROW),
            (
                Self::NPC_DISINTEGRATE,
                Self::NPC | Self::SPELLS | Self::REGEN,
            ),
            (Self::NPC_FIRE_STORM, Self::NPC | Self::SPELLS | Self::REGEN),
            (Self::GREATER_HASTE, Self::TIMED | Self::SPELLS),
            (Self::HASTE_CANDIDATE, Self::TIMED | Self::SPELLS),
            (Self::SLOW_CANDIDATE, Self::TIMED | Self::SPELLS),
            (Self::PLAYER_HEALING_CANDIDATE, Self::SPELLS | Self::WEIGHT),
            (Self::ARMOR_PROTECTION_CANDIDATE, Self::COMBAT),
        ] {
            if self.has(feature) && !self.has(dependencies) {
                return Err(invalid());
            }
        }
        if self.has(Self::HONOR) && mode != 5
            || self.has(Self::NPC)
                != matches!(
                    self.ruleset,
                    MatchRuleset::NpcEncounter | MatchRuleset::CooperativeTimeAttack
                )
            || self.has(Self::TIME_ATTACK) != (self.ruleset == MatchRuleset::CooperativeTimeAttack)
            || self.has(Self::SIEGE) != (self.ruleset == MatchRuleset::SiegeObjectives)
            || self.has(Self::PET_RACE) != (self.ruleset == MatchRuleset::PetRace)
            || (self.has(Self::FIRE_STORM) || self.has(Self::DISINTEGRATE))
                && mode != 1
                && !self.has(Self::HONOR)
            || !match self.ruleset {
                MatchRuleset::SceneOnly => self.features == 0,
                MatchRuleset::PvpSkirmish => {
                    matches!(mode, 1 | 3 | 5) && self.combat() && !self.has(Self::HONOR)
                }
                MatchRuleset::DeathmatchElimination => {
                    mode == 1 && self.combat() && !self.has(Self::HONOR)
                }
                MatchRuleset::HonorTowers => mode == 5 && self.has(Self::HONOR),
                MatchRuleset::PetRace => mode == 0 && self.features == Self::PET_RACE,
                MatchRuleset::NpcEncounter => {
                    mode == 2
                        && self.combat()
                        && self.features
                            & !(Self::NPC
                                | Self::COMBAT
                                | Self::SPELLS
                                | Self::ITEMS
                                | Self::WEIGHT
                                | Self::REGEN
                                | Self::ISOLATED_FORCES
                                | Self::QUEST_OBJECTIVES
                                | Self::QUEST_LIVES
                                | Self::NPC_LIGHT_ARROW
                                | Self::NPC_DISINTEGRATE
                                | Self::NPC_FIRE_STORM
                                | Self::TIMED
                                | Self::HASTE_CANDIDATE
                                | Self::SLOW_CANDIDATE
                                | Self::PLAYER_HEALING_CANDIDATE
                                | Self::ARMOR_PROTECTION_CANDIDATE)
                            == 0
                        && self.has(Self::TIMED)
                            == (self.has(Self::HASTE_CANDIDATE) || self.has(Self::SLOW_CANDIDATE))
                }
                MatchRuleset::CooperativeTimeAttack => {
                    mode == 4
                        && self.combat()
                        && self.has(Self::NPC)
                        && self.active_timeout_millis <= 60_000
                        && self.features
                            & !(Self::TIME_ATTACK
                                | Self::NPC
                                | Self::COMBAT
                                | Self::SPELLS
                                | Self::ITEMS
                                | Self::WEIGHT
                                | Self::REGEN
                                | Self::NPC_LIGHT_ARROW
                                | Self::NPC_DISINTEGRATE
                                | Self::NPC_FIRE_STORM
                                | Self::TIMED
                                | Self::HASTE_CANDIDATE
                                | Self::SLOW_CANDIDATE
                                | Self::PLAYER_HEALING_CANDIDATE
                                | Self::ARMOR_PROTECTION_CANDIDATE)
                            == 0
                        && self.has(Self::TIMED)
                            == (self.has(Self::HASTE_CANDIDATE) || self.has(Self::SLOW_CANDIDATE))
                }
                MatchRuleset::SiegeObjectives => {
                    mode == 3
                        && self.combat()
                        && self.features
                            & !(Self::SIEGE
                                | Self::COMBAT
                                | Self::SPELLS
                                | Self::ITEMS
                                | Self::WEIGHT
                                | Self::REGEN
                                | Self::PLAYER_HEALING_CANDIDATE
                                | Self::ARMOR_PROTECTION_CANDIDATE
                                | if self.has(Self::SLOW_CANDIDATE) {
                                    Self::TIMED
                                        | Self::SLOW_CANDIDATE
                                        | Self::HASTE_CANDIDATE
                                        | Self::GREATER_HASTE
                                } else {
                                    0
                                })
                            == 0
                }
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn capabilities(&self) -> u32 {
        Self::feature_capabilities(self.features)
    }
    /// Shared feature-to-capability mapping for legacy and canonical reads.
    pub(crate) fn feature_capabilities(features: u32) -> u32 {
        use super::protocol as p;
        let mut required = p::CAP_WORLD;
        for (feature, capability) in [
            (Self::COMBAT, p::CAP_COMBAT),
            (Self::SPELLS, p::CAP_COMBAT),
            (Self::LIGHT_ARROW, p::CAP_COMBAT),
            (Self::ITEMS, p::CAP_SHOP),
            (Self::WEIGHT, p::CAP_WEIGHT),
            (Self::TIMED, p::CAP_TIMED),
            (
                Self::SLOW_CANDIDATE,
                p::CAP_SLOW_DATA
                    | p::CAP_TIMED
                    | p::CAP_BATTLE_DATA
                    | p::CAP_ACTION_LIFE
                    | p::CAP_MATCH_DESCRIPTOR,
            ),
            (
                Self::HASTE_CANDIDATE,
                p::CAP_HASTE
                    | p::CAP_TIMED
                    | p::CAP_BATTLE_DATA
                    | p::CAP_ACTION_LIFE
                    | p::CAP_MATCH_DESCRIPTOR,
            ),
            (
                Self::GREATER_HASTE,
                p::CAP_GREATER_HASTE
                    | p::CAP_TIMED
                    | p::CAP_BATTLE_DATA
                    | p::CAP_ACTION_LIFE
                    | p::CAP_MATCH_DESCRIPTOR,
            ),
            (Self::AMMO, p::CAP_ARROW),
            (Self::REGEN, p::CAP_REGEN),
            (Self::HONOR, p::CAP_HONOR),
            (Self::FIRE_STORM, p::CAP_FIRE_STORM),
            (Self::PET_RACE, p::CAP_PET_RACE | p::CAP_MATCH_DESCRIPTOR),
            (Self::NPC, p::CAP_NPC | p::CAP_MATCH_DESCRIPTOR),
            (Self::SIEGE, p::CAP_MATCH_DESCRIPTOR),
            (
                Self::DISINTEGRATE,
                p::CAP_DISINTEGRATE | p::CAP_MATCH_DESCRIPTOR,
            ),
            (
                Self::NPC_DISINTEGRATE,
                p::CAP_NPC_DISINTEGRATE | p::CAP_NPC | p::CAP_MATCH_DESCRIPTOR,
            ),
            (
                Self::NPC_FIRE_STORM,
                p::CAP_NPC_FIRE_STORM | p::CAP_NPC | p::CAP_REGEN | p::CAP_MATCH_DESCRIPTOR,
            ),
        ] {
            if features & feature == feature {
                required |= capability;
            }
        }
        required
    }
    fn decode(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        if r.u16()? != 1 {
            return Err(invalid());
        }
        let ruleset = match r.u8()? {
            0 => MatchRuleset::SceneOnly,
            1 => MatchRuleset::PvpSkirmish,
            2 => MatchRuleset::DeathmatchElimination,
            3 => MatchRuleset::HonorTowers,
            4 => MatchRuleset::PetRace,
            5 => MatchRuleset::NpcEncounter,
            6 => MatchRuleset::CooperativeTimeAttack,
            7 => MatchRuleset::SiegeObjectives,
            _ => return Err(invalid()),
        };
        r.zero(1)?; // Legacy timeout byte, still zero; dedicated rules stay in their appendix.
        let value = Self {
            ruleset,
            features: r.u32()?,
            preparation_timeout_millis: r.u32()?,
            load_timeout_millis: r.u32()?,
            countdown_nanos: r.u64()?,
            active_timeout_millis: r.u32()?,
        };
        r.zero(4)?; // ServerMonotonic clock kind 0, then three reserved bytes.
        Ok(value)
    }
    #[cfg(test)]
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend(1_u16.to_be_bytes());
        out.extend([self.ruleset as u8, 0]);
        out.extend(self.features.to_be_bytes());
        out.extend(self.preparation_timeout_millis.to_be_bytes());
        out.extend(self.load_timeout_millis.to_be_bytes());
        out.extend(self.countdown_nanos.to_be_bytes());
        out.extend(self.active_timeout_millis.to_be_bytes());
        out.extend([0; 4]);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Prepare,
    Loaded,
    Move,
    Leave,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok,
    Denied,
    Busy,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None,
    InvalidState,
    NotOwner,
    RoomRevision,
    NotReady,
    NoBinding,
    Capacity,
    PreparationFailed,
    NotActive,
    StalePosition,
    TooFast,
    Blocked,
    Occupied,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Preparing,
    Loading,
    Countdown,
    Active,
    Aborted,
    Finished,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SettlementState {
    Pending,
    Saved,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Verdict {
    Win,
    Loss,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Participant {
    pub kind: MemberKind,
    pub entity_id: u64,
    pub key: CharacterKey,
    pub team: u8,
    pub verdict: Verdict,
}
impl Participant {
    pub fn identity(&self) -> ParticipantId {
        ParticipantId::new(self.kind, self.key)
    }
    pub fn matches_character(&self, key: CharacterKey) -> bool {
        self.identity().matches_character(key)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchResult {
    pub result_id: [u8; 16],
    pub match_id: u64,
    pub generation: u64,
    pub engine_revision: i64,
    pub finished_at_millis: i64,
    pub mode: u8,
    pub map_id: u32,
    pub participants: Vec<Participant>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub event: MatchResult,
    pub state: SettlementState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AbortReason {
    None,
    Disconnected,
    LoadTimeout,
    ActiveTimeout,
    ServerAbort,
    ClockRegressed,
    PreparationCancelled,
    PreparationFailed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub action: Action,
    pub heading: u8,
    pub room_id: u64,
    pub expected_room_revision: i64,
    pub match_id: u64,
    pub generation: u64,
    pub sequence: u64,
    pub from_x: i32,
    pub from_y: i32,
    pub manifest_digest: [u8; 32],
}
impl Request {
    pub fn read(context: SelectionContext, channel_id: u8) -> Self {
        Self {
            context,
            channel_id,
            action: Action::Read,
            heading: 0,
            room_id: 0,
            expected_room_revision: 0,
            match_id: 0,
            generation: 0,
            sequence: 0,
            from_x: 0,
            from_y: 0,
            manifest_digest: [0; 32],
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        let zero_move =
            self.heading == 0 && self.sequence == 0 && self.from_x == 0 && self.from_y == 0;
        let zero_room = self.room_id == 0 && self.expected_room_revision == 0;
        let zero_hash = self.manifest_digest == [0; 32];
        let matched = positive(self.match_id) && positive(self.generation);
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || self.heading > 7
            || !match self.action {
                Action::Read => {
                    zero_room
                        && zero_move
                        && zero_hash
                        && (matched || self.match_id == 0 && self.generation == 0)
                }
                Action::Prepare => {
                    positive(self.room_id)
                        && self.expected_room_revision > 0
                        && self.match_id == 0
                        && self.generation == 0
                        && zero_move
                        && zero_hash
                }
                Action::Loaded => zero_room && matched && zero_move && !zero_hash,
                Action::Move => zero_room && matched && positive(self.sequence) && zero_hash,
                Action::Leave => zero_room && matched && zero_move && zero_hash,
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sector {
    pub x: u16,
    pub y: u16,
    pub byte_length: u32,
    pub sha256: [u8; 32],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub version: u16,
    pub map_id: u32,
    pub digest: [u8; 32],
    pub sectors: Vec<Sector>,
}
impl Manifest {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if self.version != 1
            || self.map_id > i32::MAX as u32
            || self.digest == [0; 32]
            || self.sectors.is_empty()
            || self.sectors.len() > super::world_map::MAX_SECTORS
        {
            return Err(invalid());
        }
        let mut previous = None;
        let mut total = 0_u64;
        for s in &self.sectors {
            let key = (s.x, s.y);
            if !(49158..=8 * 1024 * 1024).contains(&s.byte_length)
                || s.sha256 == [0; 32]
                || previous.is_some_and(|p| p >= key)
            {
                return Err(invalid());
            }
            total += u64::from(s.byte_length);
            if total > 32 * 1024 * 1024 {
                return Err(invalid());
            }
            previous = Some(key);
        }
        Ok(())
    }
    pub fn contains(&self, cell: WorldCell) -> bool {
        self.sectors.iter().any(|s| {
            let x = 32704 + (i64::from(s.x) - 0x7fff) * 64;
            let y = 32704 + (i64::from(s.y) - 0x7fff) * 64;
            (x..x + 64).contains(&i64::from(cell.x)) && (y..y + 64).contains(&i64::from(cell.y))
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub kind: MemberKind,
    pub entity_id: u64,
    pub key: CharacterKey,
    pub position: WorldCell,
    pub heading: u8,
    pub loaded: bool,
    pub sequence: u64,
    pub step_interval_nanos: u64,
}
impl Actor {
    pub fn identity(&self) -> ParticipantId {
        ParticipantId::new(self.kind, self.key)
    }
    pub fn matches_character(&self, key: CharacterKey) -> bool {
        self.identity().matches_character(key)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub match_id: u64,
    pub generation: u64,
    pub revision: i64,
    pub phase: Phase,
    pub abort_reason: AbortReason,
    pub room: rooms::RoomSnapshot,
    pub manifest: Manifest,
    pub actors: Vec<Actor>,
    pub completion: Option<Completion>,
    /// v4 only. Empty means this match has no spell authority, not an empty learned list.
    pub casts: Vec<super::spell_protocol::CastState>,
    /// None is the legacy homogeneous-session contract, never inferred from a missing Combat reply.
    pub descriptor: Option<MatchDescriptor>,
    /// v16 independent race progress/result; never a two-team win/loss event.
    pub race: Option<super::race_protocol::RaceData>,
    /// v20: stable global actor IDs, but only this force's world state.
    /// None is a shared scene; an isolated descriptor must never omit its lane.
    pub audience_lane: Option<u8>,
    /// v21 own-force objectives plus a position-free public terminal proof.
    pub quest: Option<super::quest_protocol::Data>,
    /// v25 cooperative authority; never an old two-team completion event.
    pub time_attack: Option<super::time_attack_protocol::Data>,
    /// v26 dedicated siege objectives, dynamic geometry, lives and durable receipt.
    pub siege: Option<super::siege_protocol::Data>,
}
impl Frame {
    pub fn actor(&self, entity_id: u64) -> Option<&Actor> {
        self.actors.iter().find(|a| a.entity_id == entity_id)
    }
    pub fn member(&self, entity_id: u64) -> Option<&rooms::Member> {
        self.room
            .members
            .get(usize::try_from(entity_id.checked_sub(1)?).ok()?)
    }
    pub fn combat_enabled(&self, legacy_supported: bool) -> bool {
        self.descriptor
            .as_ref()
            .map_or(legacy_supported, MatchDescriptor::combat)
    }
    pub fn feature_enabled(&self, feature: u32, legacy_supported: bool) -> bool {
        self.descriptor
            .as_ref()
            .map_or(legacy_supported, |d| d.has(feature))
    }
    pub(crate) fn feature_presence_matches(
        &self,
        feature: u32,
        present: bool,
        legacy_supported: bool,
    ) -> bool {
        self.descriptor
            .as_ref()
            .map_or(present == legacy_supported, |d| {
                (!d.has(feature) || legacy_supported) && present == d.has(feature)
            })
    }
    pub fn cast(&self, entity_id: u64) -> Option<&super::spell_protocol::CastState> {
        self.casts.iter().find(|c| c.entity_id == entity_id)
    }
    pub fn validate(&self, self_key: CharacterKey) -> Result<(), ApplicationError> {
        self.room.validate_members(self_key)?;
        self.manifest.validate()?;
        super::time_attack_protocol::validate_world(self)?;
        super::siege_protocol::validate_world(self)?;
        if self.race.is_some()
            != self
                .descriptor
                .is_some_and(|d| d.ruleset == MatchRuleset::PetRace)
        {
            return Err(invalid());
        }
        if let Some(race) = &self.race {
            race.validate(self)?;
        }
        if let Some(descriptor) = &self.descriptor {
            descriptor.validate(self.room.summary.settings.mode)?;
            if descriptor.has(MatchDescriptor::SPELLS) != !self.casts.is_empty()
                || self.completion.is_some() && !descriptor.results()
            {
                return Err(invalid());
            }
        }
        if !self.casts.is_empty() {
            if self.casts.len() != self.actors.len() {
                return Err(invalid());
            }
            for (cast, actor) in self.casts.iter().zip(&self.actors) {
                if cast.entity_id != actor.entity_id {
                    return Err(invalid());
                }
                cast.validate(self)?;
                if self.phase == Phase::Preparing
                    && *cast
                        != super::spell_protocol::CastState::initial(
                            actor.entity_id,
                            cast.cast_interval_millis,
                        )
                {
                    return Err(invalid());
                }
            }
        }
        let room = &self.room;
        let isolated = self
            .descriptor
            .is_some_and(|d| d.has(MatchDescriptor::ISOLATED_FORCES));
        if isolated != self.audience_lane.is_some()
            || self.audience_lane.is_some_and(|lane| lane > 1)
            || !self.actors.iter().any(|a| a.matches_character(self_key))
        {
            return Err(invalid());
        }
        let expected = room
            .members
            .iter()
            .filter(|m| self.audience_lane.is_none_or(|lane| m.team == lane))
            .count();
        if !positive(self.match_id)
            || !positive(self.generation)
            || self.revision < 1
            || self.generation != room.summary.revision as u64
            || (self.phase == Phase::Aborted) != (self.abort_reason != AbortReason::None)
            || (self.phase == Phase::Finished)
                != (self.completion.is_some()
                    || self
                        .race
                        .as_ref()
                        .is_some_and(|r| r.state == super::race_protocol::RaceState::Complete)
                    || self
                        .time_attack
                        .as_ref()
                        .is_some_and(|d| d.phase == super::time_attack_protocol::Phase::Finished)
                    || self
                        .siege
                        .as_ref()
                        .is_some_and(|d| d.phase == super::siege_protocol::Phase::Finished))
            || !room.summary.satisfied
            || room.summary.ready != room.summary.users
            || self.actors.is_empty()
            || self.actors.len() > 100
            || self.actors.len() != expected
        {
            return Err(invalid());
        }
        let first = room.members.iter().filter(|m| m.team == 0).count();
        let second = room.members.len() - first;
        if (room.summary.settings.mode != 4 && room.members.len() < 2)
            || (matches!(room.summary.settings.mode, 1 | 2 | 3 | 5)
                && (first == 0
                    || second == 0
                    || room.summary.settings.attributes & 32 != 0 && first != second
                    || room.summary.settings.mode == 3
                        && (!(15..=50).contains(&first) || !(15..=50).contains(&second))))
        {
            return Err(invalid());
        }
        let mut positions = std::collections::HashSet::new();
        let mut previous = 0;
        for (i, a) in self.actors.iter().enumerate() {
            let member = self.member(a.entity_id).ok_or_else(invalid)?;
            if a.entity_id <= previous
                || self.audience_lane.is_none() && a.entity_id != i as u64 + 1
                || a.identity() != member.identity()
                || self.audience_lane.is_some_and(|lane| member.team != lane)
                || a.heading > 7
                || a.sequence > i64::MAX as u64
                || !(1_000_000..=10_000_000_000).contains(&a.step_interval_nanos)
                || self.actor_occupies(a.entity_id)
                    && !positions.insert((a.position.x, a.position.y))
                || !self.manifest.contains(a.position)
                || self.phase == Phase::Preparing && (a.loaded || a.sequence != 0)
                || matches!(
                    self.phase,
                    Phase::Countdown | Phase::Active | Phase::Finished
                ) && !a.loaded
            {
                return Err(invalid());
            }
            previous = a.entity_id;
        }
        if let Some(c) = &self.completion {
            let e = &c.event;
            if e.result_id == [0; 16]
                || e.match_id != self.match_id
                || e.generation != self.generation
                || e.engine_revision < 1
                || e.engine_revision >= self.revision
                || e.finished_at_millis < 0
                || !matches!(e.mode, 1 | 2 | 5)
                || e.mode != room.summary.settings.mode
                || e.map_id != room.summary.settings.map_id
                || e.participants.len() != room.members.len()
                || self
                    .casts
                    .iter()
                    .any(|cast| cast.cast_revision > e.engine_revision.saturating_add(1))
            {
                return Err(invalid());
            }
            let mut winner = None;
            let mut loser = None;
            for (i, (p, m)) in e.participants.iter().zip(&room.members).enumerate() {
                if p.entity_id != i as u64 + 1 || p.identity() != m.identity() || p.team != m.team {
                    return Err(invalid());
                }
                let team = if p.verdict == Verdict::Win {
                    &mut winner
                } else {
                    &mut loser
                };
                if team.is_some_and(|old| old != p.team) {
                    return Err(invalid());
                }
                *team = Some(p.team);
            }
            if winner.is_none() || loser.is_none() || winner == loser {
                return Err(invalid());
            }
        }
        super::quest_protocol::validate(self)?;
        Ok(())
    }
    pub fn actor_occupies(&self, entity_id: u64) -> bool {
        self.time_attack
            .as_ref()
            .is_none_or(|d| !d.departed(entity_id))
            && self
                .race
                .as_ref()
                .is_none_or(|race| race.competing(entity_id))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: Status,
    pub reason: Reason,
    pub frame: Option<Frame>,
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    NotRequested,
    Unsupported,
    Ready(std::sync::Arc<Reply>),
}
impl State {
    pub fn frame(&self) -> Option<&Frame> {
        if let Self::Ready(reply) = self {
            reply.frame.as_ref()
        } else {
            None
        }
    }
}
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        let r = &self.echo;
        r.validate()?;
        if (self.status == Status::Ok) != (self.reason == Reason::None) {
            return Err(invalid());
        }
        if let Some(f) = &self.frame {
            f.validate(r.context.key)?;
            if r.match_id != 0 && (r.match_id != f.match_id || r.generation != f.generation) {
                return Err(invalid());
            }
        }
        if self.status != Status::Ok {
            return Ok(());
        }
        match r.action {
            Action::Read => {}
            Action::Leave => {
                if self.frame.is_some() {
                    return Err(invalid());
                }
            }
            Action::Prepare => {
                let f = self.frame.as_ref().ok_or_else(invalid)?;
                if f.room.summary.room_id != r.room_id
                    || Some(f.generation as i64) != r.expected_room_revision.checked_add(1)
                    || f.room.owner != r.context.key
                {
                    return Err(invalid());
                }
            }
            Action::Loaded => {
                let f = self.frame.as_ref().ok_or_else(invalid)?;
                if f.phase == Phase::Preparing
                    || f.manifest.digest != r.manifest_digest
                    || !f
                        .actors
                        .iter()
                        .any(|a| a.matches_character(r.context.key) && a.loaded)
                {
                    return Err(invalid());
                }
            }
            Action::Move => {
                let f = self.frame.as_ref().ok_or_else(invalid)?;
                let a = f
                    .actors
                    .iter()
                    .find(|a| a.matches_character(r.context.key))
                    .ok_or_else(invalid)?;
                let dx = [0, 1, 1, 1, 0, -1, -1, -1];
                let dy = [-1, -1, 0, 1, 1, 1, 0, -1];
                if f.phase != Phase::Active
                    || a.sequence != r.sequence
                    || a.heading != r.heading
                    || i64::from(a.position.x) != i64::from(r.from_x) + dx[r.heading as usize]
                    || i64::from(a.position.y) != i64::from(r.from_y) + dy[r.heading as usize]
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
    pub fn confirms(&self, request: &Request) -> bool {
        self.echo == *request && self.validate().is_ok()
    }
}
/// Monotonic snapshots for one fixed selection/channel and assigned match.
/// Only PREPARING->LOADING may replace tentative movement timing; the frozen roster stays identical.
pub fn follows(prior: Option<&Reply>, next: &Reply) -> bool {
    if next.validate().is_err() {
        return false;
    }
    let Some(old) = prior else {
        return true;
    };
    if old.echo.context != next.echo.context || old.echo.channel_id != next.echo.channel_id {
        return false;
    }
    if old.frame.is_some() && next.frame.is_none() {
        return next.status == Status::Ok && next.echo.action == Action::Leave;
    }
    let (Some(a), Some(b)) = (&old.frame, &next.frame) else {
        return true;
    };
    if a.match_id != b.match_id
        || a.generation != b.generation
        || a.manifest != b.manifest
        || a.room != b.room
        || a.descriptor != b.descriptor
        || a.audience_lane != b.audience_lane
        || a.actors.len() != b.actors.len()
        || b.revision < a.revision
        || !super::spell_protocol::casts_follow(a, b)
        || !super::race_protocol::follows(a, b)
        || !super::quest_protocol::follows(a, b)
        || !super::time_attack_protocol::follows(a, b)
        || !super::siege_protocol::follows(a, b)
    {
        return false;
    }
    if b.revision == a.revision {
        return a == b;
    }
    if !terminal_follows(a, b) {
        return false;
    }
    let qualifying = a.phase == Phase::Preparing;
    a.actors.iter().zip(&b.actors).all(|(old, new)| {
        old.entity_id == new.entity_id
            && old.identity() == new.identity()
            && new.sequence >= old.sequence
            && (!old.loaded || new.loaded)
            && (qualifying || old.step_interval_nanos == new.step_interval_nanos)
            && (old.sequence != new.sequence
                || old.heading == new.heading
                    && if super::siege_protocol::respawn_is_latest_relocation(a, b, old.entity_id) {
                        true // Siege follow checks the configured role camp.
                    } else if super::spell_protocol::teleported(a, b, old.entity_id) {
                        b.cast(new.entity_id)
                            .is_some_and(|cast| new.position == cast.to)
                    } else {
                        old.position == new.position
                    })
    })
}
/// Terminal outcomes never change. Only PENDING -> SAVED/FAILED may add one
/// world revision after FINISHED; positions and the result event remain exact.
pub(crate) fn terminal_follows(a: &Frame, b: &Frame) -> bool {
    if a.phase == Phase::Aborted {
        return false;
    }
    if a.phase == Phase::Finished {
        if a.siege.is_some() {
            return super::siege_protocol::terminal_receipt_follows(a, b);
        }
        if a.time_attack.is_some() {
            return super::time_attack_protocol::terminal_receipt_follows(a, b);
        }
        if a.race.is_some() {
            return super::race_protocol::terminal_receipt_follows(a, b);
        }
        let (Some(old), Some(new)) = (&a.completion, &b.completion) else {
            return false;
        };
        return b.phase == Phase::Finished
            && a.actors == b.actors
            && a.casts == b.casts
            && old.event == new.event
            && old.state == SettlementState::Pending
            && new.state != SettlementState::Pending
            && a.revision.checked_add(1) == Some(b.revision);
    }
    if b.phase == Phase::Finished {
        return true;
    }
    b.phase == Phase::Aborted || (b.phase as u8) >= (a.phase as u8)
}
pub fn encode_request(id: u32, request: &Request) -> Result<Vec<u8>, ApplicationError> {
    request.validate()?;
    let mut out = app::header(id, REQUEST_KIND, 16 + REQUEST_BYTES)?;
    put_request(&mut out, request);
    Ok(out)
}
fn put_request(out: &mut Vec<u8>, r: &Request) {
    out.extend(r.context.key.source_id.to_be_bytes());
    out.extend(r.context.key.object_id.to_be_bytes());
    out.extend(r.context.epoch.to_be_bytes());
    out.extend([r.channel_id, r.action as u8, r.heading, 0]);
    out.extend(r.room_id.to_be_bytes());
    out.extend(r.expected_room_revision.to_be_bytes());
    out.extend(r.match_id.to_be_bytes());
    out.extend(r.generation.to_be_bytes());
    out.extend(r.sequence.to_be_bytes());
    out.extend(r.from_x.to_be_bytes());
    out.extend(r.from_y.to_be_bytes());
    out.extend(r.manifest_digest);
}
fn positive(n: u64) -> bool {
    n > 0 && n <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid world frame")
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(invalid)?;
        let v = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
        self.pos = end;
        Ok(v)
    }
    fn u8(&mut self) -> Result<u8, ApplicationError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, ApplicationError> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, ApplicationError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, ApplicationError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i64(&mut self) -> Result<i64, ApplicationError> {
        Ok(self.u64()? as i64)
    }
    fn i32(&mut self) -> Result<i32, ApplicationError> {
        Ok(self.u32()? as i32)
    }
    fn digest(&mut self) -> Result<[u8; 32], ApplicationError> {
        Ok(self.take(32)?.try_into().unwrap())
    }
    fn zero(&mut self, n: usize) -> Result<(), ApplicationError> {
        if self.take(n)?.iter().any(|b| *b != 0) {
            return Err(invalid());
        }
        Ok(())
    }
    fn boolean(&mut self) -> Result<bool, ApplicationError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid()),
        }
    }
    fn request(&mut self) -> Result<Request, ApplicationError> {
        let context = SelectionContext {
            key: CharacterKey {
                source_id: self.u32()?,
                object_id: self.u64()?,
            },
            epoch: self.u64()?,
        };
        let channel_id = self.u8()?;
        let action = match self.u8()? {
            0 => Action::Read,
            1 => Action::Prepare,
            2 => Action::Loaded,
            3 => Action::Move,
            4 => Action::Leave,
            _ => return Err(invalid()),
        };
        let heading = self.u8()?;
        self.zero(1)?;
        let r = Request {
            context,
            channel_id,
            action,
            heading,
            room_id: self.u64()?,
            expected_room_revision: self.i64()?,
            match_id: self.u64()?,
            generation: self.u64()?,
            sequence: self.u64()?,
            from_x: self.i32()?,
            from_y: self.i32()?,
            manifest_digest: self.digest()?,
        };
        r.validate()?;
        Ok(r)
    }
}
#[cfg(test)]
pub(crate) fn decode_body(bytes: &[u8]) -> Result<Reply, ApplicationError> {
    decode_body_version(bytes, 1)
}
pub(crate) fn decode_body_version(
    bytes: &[u8],
    protocol_version: u16,
) -> Result<Reply, ApplicationError> {
    if bytes.len() > MAX_FRAME_BYTES - 16
        || !(1..=super::protocol::IMPLEMENTED_WORLD_VERSION).contains(&protocol_version)
    {
        return Err(invalid());
    }
    let mut r = Reader { bytes, pos: 0 };
    let echo = r.request()?;
    let status = match r.u8()? {
        0 => Status::Ok,
        1 => Status::Denied,
        2 => Status::Busy,
        3 => Status::Unavailable,
        _ => return Err(invalid()),
    };
    let reason = match r.u8()? {
        0 => Reason::None,
        1 => Reason::InvalidState,
        2 => Reason::NotOwner,
        3 => Reason::RoomRevision,
        4 => Reason::NotReady,
        5 => Reason::NoBinding,
        6 => Reason::Capacity,
        7 => Reason::PreparationFailed,
        8 => Reason::NotActive,
        9 => Reason::StalePosition,
        10 => Reason::TooFast,
        11 => Reason::Blocked,
        12 => Reason::Occupied,
        _ => return Err(invalid()),
    };
    let present = r.boolean()?;
    r.zero(1)?;
    let frame = if present {
        let (frame, used) =
            decode_frame_version(&bytes[r.pos..], echo.context.key, protocol_version)?;
        r.take(used)?;
        Some(frame)
    } else {
        None
    };
    if r.pos != bytes.len() {
        return Err(invalid());
    }
    let reply = Reply {
        echo,
        status,
        reason,
        frame,
    };
    reply.validate()?;
    Ok(reply)
}
pub(crate) fn decode_frame_version(
    bytes: &[u8],
    self_key: CharacterKey,
    protocol_version: u16,
) -> Result<(Frame, usize), ApplicationError> {
    if !(1..=CODEC_MAX_VERSION).contains(&protocol_version) {
        return Err(invalid());
    }
    let mut r = Reader { bytes, pos: 0 };
    let match_id = r.u64()?;
    let generation = r.u64()?;
    let revision = r.i64()?;
    let phase = match r.u8()? {
        0 => Phase::Preparing,
        1 => Phase::Loading,
        2 => Phase::Countdown,
        3 => Phase::Active,
        4 => Phase::Aborted,
        5 if protocol_version >= 3 => Phase::Finished,
        _ => return Err(invalid()),
    };
    let abort_reason = match r.u8()? {
        0 => AbortReason::None,
        1 => AbortReason::Disconnected,
        2 => AbortReason::LoadTimeout,
        3 => AbortReason::ActiveTimeout,
        4 => AbortReason::ServerAbort,
        5 => AbortReason::ClockRegressed,
        6 => AbortReason::PreparationCancelled,
        7 => AbortReason::PreparationFailed,
        _ => return Err(invalid()),
    };
    r.zero(2)?;
    let (room, used) = rooms::decode_joined_version(&bytes[r.pos..], protocol_version)?;
    r.take(used)?;
    let version = r.u16()?;
    let count = r.u16()? as usize;
    let map_id = r.u32()?;
    let digest = r.digest()?;
    if count == 0 || count > super::world_map::MAX_SECTORS || protocol_version < 34 && count > 16 {
        return Err(invalid());
    }
    let mut sectors = Vec::with_capacity(count);
    for _ in 0..count {
        sectors.push(Sector {
            x: r.u16()?,
            y: r.u16()?,
            byte_length: r.u32()?,
            sha256: r.digest()?,
        });
    }
    let manifest = Manifest {
        version,
        map_id,
        digest,
        sectors,
    };
    let count = r.u16()? as usize;
    r.zero(2)?;
    if count == 0 || count > 100 || protocol_version < 20 && count != room.members.len() {
        return Err(invalid());
    }
    let mut actors = Vec::with_capacity(count);
    for _ in 0..count {
        let entity_id = r.u64()?;
        let member = room
            .members
            .get(
                usize::try_from(entity_id.checked_sub(1).ok_or_else(invalid)?)
                    .map_err(|_| invalid())?,
            )
            .ok_or_else(invalid)?;
        let position = WorldCell {
            x: r.i32()?,
            y: r.i32()?,
        };
        let heading = r.u8()?;
        let loaded = r.boolean()?;
        r.zero(2)?;
        actors.push(Actor {
            kind: member.kind,
            entity_id,
            key: member.key,
            position,
            heading,
            loaded,
            sequence: r.u64()?,
            step_interval_nanos: r.u64()?,
        });
    }
    let completion = if protocol_version >= 3 {
        let present = r.boolean()?;
        r.zero(3)?;
        if present {
            let result_id = r.take(16)?.try_into().unwrap();
            let match_id = r.u64()?;
            let generation = r.u64()?;
            let engine_revision = r.i64()?;
            let finished_at_millis = r.i64()?;
            let mode = r.u8()?;
            if mode != 1
                && !(protocol_version >= 13 && mode == 5)
                && !(protocol_version >= 21 && mode == 2)
            {
                return Err(invalid());
            }
            let state = match r.u8()? {
                0 => SettlementState::Pending,
                1 => SettlementState::Saved,
                2 => SettlementState::Failed,
                _ => return Err(invalid()),
            };
            r.zero(2)?;
            let map_id = r.u32()?;
            let count = r.u16()? as usize;
            r.zero(2)?;
            if count < 2 || count > 100 || count != room.members.len() {
                return Err(invalid());
            }
            let mut participants = Vec::with_capacity(count);
            for _ in 0..count {
                let kind = if protocol_version >= 7 {
                    let kind = match r.u8()? {
                        0 => MemberKind::Human,
                        1 => MemberKind::ServerBot,
                        _ => return Err(invalid()),
                    };
                    r.zero(3)?;
                    kind
                } else {
                    MemberKind::Human
                };
                let entity_id = r.u64()?;
                let key = CharacterKey {
                    source_id: r.u32()?,
                    object_id: r.u64()?,
                };
                let team = r.u8()?;
                let verdict = match r.u8()? {
                    0 => Verdict::Win,
                    1 => Verdict::Loss,
                    _ => return Err(invalid()),
                };
                r.zero(2)?;
                participants.push(Participant {
                    kind,
                    entity_id,
                    key,
                    team,
                    verdict,
                });
            }
            Some(Completion {
                event: MatchResult {
                    result_id,
                    match_id,
                    generation,
                    engine_revision,
                    finished_at_millis,
                    mode,
                    map_id,
                    participants,
                },
                state,
            })
        } else {
            None
        }
    } else {
        None
    };
    let casts = if protocol_version >= 4 {
        let count = usize::from(r.u16()?);
        r.zero(2)?;
        if count != 0 && count != actors.len() || count > 100 || bytes.len() - r.pos < count * 64 {
            return Err(invalid());
        }
        let mut rows = Vec::with_capacity(count);
        for _ in 0..count {
            let row = super::spell_protocol::CastState {
                entity_id: r.u64()?,
                request_sequence: r.u64()?,
                cast_count: r.u64()?,
                cast_revision: r.i64()?,
                mp_spent: r.i64()?,
                skill_id: r.u32()?,
                from: WorldCell {
                    x: r.i32()?,
                    y: r.i32()?,
                },
                to: WorldCell {
                    x: r.i32()?,
                    y: r.i32()?,
                },
                cast_interval_millis: r.u32()?,
            };
            rows.push(row);
        }
        rows
    } else {
        Vec::new()
    };
    let descriptor = if protocol_version >= 15 {
        let present = r.boolean()?;
        r.zero(7)?;
        if present {
            Some(MatchDescriptor::decode(&mut r)?)
        } else {
            None
        }
    } else {
        None
    };
    let race = if protocol_version >= 16 {
        let (race, consumed) = super::race_protocol::decode(&bytes[r.pos..])?;
        r.take(consumed)?;
        race
    } else {
        None
    };
    if protocol_version < 17 && descriptor.is_some_and(|d| d.has(MatchDescriptor::NPC)) {
        return Err(invalid());
    }
    if protocol_version < 23
        && descriptor.is_some_and(|d| d.has(MatchDescriptor::NPC | MatchDescriptor::SPELLS))
    {
        return Err(invalid());
    }
    if protocol_version < 24 && descriptor.is_some_and(|d| d.has(MatchDescriptor::NPC_LIGHT_ARROW))
    {
        return Err(invalid());
    }
    if protocol_version < 30
        && descriptor
            .is_some_and(|d| d.has(MatchDescriptor::TIME_ATTACK | MatchDescriptor::NPC_LIGHT_ARROW))
    {
        return Err(invalid());
    }
    if protocol_version < 27 && descriptor.is_some_and(|d| d.has(MatchDescriptor::DISINTEGRATE)) {
        return Err(invalid());
    }
    if protocol_version < 29 && descriptor.is_some_and(|d| d.has(MatchDescriptor::NPC_DISINTEGRATE))
    {
        return Err(invalid());
    }
    if protocol_version < 31 && descriptor.is_some_and(|d| d.has(MatchDescriptor::NPC_FIRE_STORM)) {
        return Err(invalid());
    }
    let audience_lane = if protocol_version >= 20 {
        let lane = match r.u8()? {
            0 => None,
            1 => Some(0),
            2 => Some(1),
            _ => return Err(invalid()),
        };
        r.zero(3)?;
        lane
    } else {
        None
    };
    let quest = if protocol_version >= 21 {
        let (quest, used) =
            super::quest_protocol::decode_version(&bytes[r.pos..], protocol_version)?;
        r.take(used)?;
        quest
    } else {
        None
    };
    let frame = Frame {
        match_id,
        generation,
        revision,
        phase,
        abort_reason,
        room,
        manifest,
        actors,
        completion,
        casts,
        descriptor,
        race,
        audience_lane,
        quest,
        time_attack: if protocol_version >= 25 {
            let fire = descriptor.is_some_and(|d| d.has(MatchDescriptor::NPC_FIRE_STORM));
            let (value, used) = if fire {
                super::time_attack_protocol::decode_with_fire(&bytes[r.pos..], true)?
            } else {
                super::time_attack_protocol::decode(&bytes[r.pos..])?
            };
            r.take(used)?;
            value
        } else {
            None
        },
        siege: if protocol_version >= 26 {
            let (value, used) = super::siege_protocol::decode(&bytes[r.pos..])?;
            r.take(used)?;
            value
        } else {
            None
        },
    };
    frame.validate(self_key)?;
    Ok((frame, r.pos))
}

/// The new descriptor is mandatory iff negotiated, including the first
/// PREPARING frame. Source/Shop/Equipment payloads are intentionally not here.
pub(crate) fn descriptor_negotiated(response: &app::Response, ack: &super::protocol::Ack) -> bool {
    // Explicit opt-in only, including a no-frame denial. Codec support alone
    // never grants a new spell or encounter to an older negotiated session.
    if matches!(response, app::Response::NpcFireStorm(_))
        && !super::npc_fire_storm_protocol::negotiated(ack)
    {
        return false;
    }
    if matches!(response, app::Response::NpcDisintegrate(_))
        && !super::npc_disintegrate_protocol::negotiated(ack)
    {
        return false;
    }
    if matches!(response, app::Response::Disintegrate(_))
        && !super::disintegrate_cast_protocol::negotiated(ack)
    {
        return false;
    }
    if matches!(response, app::Response::Siege(_))
        && (ack.version < super::siege_protocol::VERSION
            || ack.capabilities & super::protocol::CAP_MATCH_DESCRIPTOR == 0)
    {
        return false;
    }
    if matches!(response, app::Response::NpcSpell(_))
        && (ack.version < 24 || ack.capabilities & super::protocol::CAP_NPC == 0)
    {
        return false;
    }
    if matches!(response, app::Response::Npc(_))
        && (ack.version < 17 || ack.capabilities & super::protocol::CAP_NPC == 0)
    {
        return false;
    }
    let frame = match response {
        app::Response::World(r) => r.frame.as_ref(),
        app::Response::Combat(r) => r.frame.as_ref().map(|f| &f.world),
        app::Response::Siege(r) => r.frame.as_ref().map(|f| &f.world),
        app::Response::Cast(r) => r.frame.as_ref().map(|f| &f.world),
        app::Response::Disintegrate(r) => r.frame.as_ref().map(|f| &f.world),
        app::Response::Item(r) => r.frame.as_ref().map(|f| &f.world),
        app::Response::Ammo(r) => r.frame.as_ref().map(|f| &f.world),
        app::Response::Npc(r) => r.frame.as_ref().map(|f| &f.combat.world),
        app::Response::NpcSpell(r) => r.frame.as_ref().map(|f| &f.combat.world),
        app::Response::NpcDisintegrate(r) => r.frame.as_ref().map(|f| &f.combat.world),
        app::Response::NpcFireStorm(r) => r.frame.as_ref().map(|f| &f.combat.world),
        _ => return true,
    };
    frame.is_none_or(|f| {
        if f.manifest.sectors.len() > 16
            && (ack.version < 34 || ack.capabilities & super::protocol::CAP_LARGE_MAPS == 0)
        {
            return false;
        }
        let enabled =
            ack.version >= 15 && ack.capabilities & super::protocol::CAP_MATCH_DESCRIPTOR != 0;
        (f.descriptor
            .is_none_or(|d| !d.has(MatchDescriptor::NPC_FIRE_STORM))
            || super::npc_fire_storm_protocol::negotiated(ack))
            && f.descriptor.is_some() == enabled
            && (f.race.is_none()
                || ack.version >= 16 && ack.capabilities & super::protocol::CAP_PET_RACE != 0)
            && (f.descriptor.is_none_or(|d| !d.has(MatchDescriptor::NPC))
                || ack.version >= 17 && ack.capabilities & super::protocol::CAP_NPC != 0)
            && (f
                .descriptor
                .is_none_or(|d| !d.has(MatchDescriptor::ISOLATED_FORCES))
                || ack.version >= 20)
            && (f.quest.is_none() || ack.version >= 21)
            && (f.time_attack.is_none() || ack.version >= 25)
            && (f.descriptor.is_none_or(|d| {
                !d.has(MatchDescriptor::TIME_ATTACK | MatchDescriptor::NPC_LIGHT_ARROW)
            }) || ack.version >= 30)
            && (f.siege.is_none() || ack.version >= 26)
            && (f
                .descriptor
                .is_none_or(|d| !d.has(MatchDescriptor::DISINTEGRATE))
                || ack.version >= 27)
            && (f
                .descriptor
                .is_none_or(|d| !d.has(MatchDescriptor::NPC_DISINTEGRATE))
                || ack.version >= 29)
            && (f
                .descriptor
                .is_none_or(|d| !d.has(MatchDescriptor::NPC_LIGHT_ARROW))
                || ack.version >= 24)
            && (f.quest.as_ref().is_none_or(|q| q.lives.is_empty()) || ack.version >= 22)
            && (f
                .descriptor
                .is_none_or(|d| !d.has(MatchDescriptor::NPC | MatchDescriptor::SPELLS))
                || ack.version >= 23)
            && f.descriptor.as_ref().is_none_or(|d| {
                d.validate(f.room.summary.settings.mode).is_ok()
                    && ack.capabilities & d.capabilities() == d.capabilities()
            })
    })
}

#[cfg(test)]
#[path = "match_descriptor_protocol_tests.rs"]
mod descriptor_tests;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    const REQUEST_HEX: &str = "4c544d3100000078fedcba9800140000000000070000000000000d05ffffffffffffffff030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    const FRAME_HEX: &str = "4c544d3100000165fedcba9800150000000000070000000000000d05ffffffffffffffff030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000009000000000000000600000000000000010100000000000000000000050000000000000006000204100000021c0001520001000101000000000000070000000000000d0500010000000000070000000000000d0500000000ee6b2800000301000100000000014e000153000100010000020222222222222222222222222222222222222222222222222222222222222222227fff80000000c012111111111111111111111111111111111111111111111111111111111111111100010000000000000000000100007fd4000080100200000000000000000000000000000002faf080";
    fn hex(s: &str) -> Vec<u8> {
        s.as_bytes()
            .chunks_exact(2)
            .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
            .collect()
    }
    #[test]
    fn large_manifest_bytes_require_v34_and_the_independent_negotiated_bit() {
        use super::super::protocol as p;
        for count in [16, 17, 64, 65] {
            let mut reply = sample();
            let f = reply.frame.as_mut().unwrap();
            let first = f.manifest.sectors[0].clone();
            f.manifest.sectors = (0..count)
                .map(|i| Sector {
                    y: first.y + i as u16,
                    ..first.clone()
                })
                .collect();
            if count > 64 {
                assert!(reply.validate().is_err());
                continue;
            }
            f.descriptor = Some(MatchDescriptor {
                ruleset: MatchRuleset::SceneOnly,
                features: 0,
                preparation_timeout_millis: 3000,
                load_timeout_millis: 15000,
                countdown_nanos: 0,
                active_timeout_millis: 10000,
            });
            let bytes = wire_complete_version(&reply, 34);
            assert_eq!(decode_body_version(&bytes[16..], 34).unwrap(), reply);
            assert_eq!(decode_body_version(&bytes[16..], 33).is_ok(), count <= 16);
            let response = app::Response::World(reply);
            let mut ack = p::Ack {
                request_id: 1,
                version: 34,
                numeric_schema: 1,
                capabilities: p::RUNTIME_CAPABILITIES,
            };
            assert_eq!(descriptor_negotiated(&response, &ack), count <= 16);
            ack.capabilities |= p::CAP_LARGE_MAPS;
            assert!(descriptor_negotiated(&response, &ack));
        }
    }
    pub(super) fn sample() -> Reply {
        decode_body(&hex(FRAME_HEX)[16..]).unwrap()
    }
    fn context() -> SelectionContext {
        SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        }
    }
    const COMPLETION_TAIL: &str = concat!(
        "01000000",
        "00112233445566778899aabbccddeeff",
        "00000000000000090000000000000006000000000000000700000000000004d2",
        "010000000000020200020000",
        "0000000000000001000000070000000000000d0500000000",
        "00000000000000020000002a0000000000000d0501010000"
    );
    pub(crate) fn finished() -> Frame {
        let mut f = super::super::world_runtime::tests::frame(9);
        f.generation = 6;
        f.room.summary.revision = 6;
        f.revision = 8;
        f.phase = Phase::Finished;
        for a in &mut f.actors {
            a.loaded = true;
        }
        f.completion = Some(Completion {
            state: SettlementState::Pending,
            event: MatchResult {
                result_id: hex("00112233445566778899aabbccddeeff").try_into().unwrap(),
                match_id: 9,
                generation: 6,
                engine_revision: 7,
                finished_at_millis: 1234,
                mode: 1,
                map_id: 514,
                participants: f
                    .actors
                    .iter()
                    .zip(&f.room.members)
                    .map(|(a, m)| Participant {
                        kind: a.kind,
                        entity_id: a.entity_id,
                        key: a.key,
                        team: m.team,
                        verdict: if m.team == 0 {
                            Verdict::Win
                        } else {
                            Verdict::Loss
                        },
                    })
                    .collect(),
            },
        });
        f.validate(f.actors[0].key).unwrap();
        f
    }
    fn finished_reply() -> Reply {
        Reply {
            echo: Request::read(context(), 3),
            status: Status::Ok,
            reason: Reason::None,
            frame: Some(finished()),
        }
    }
    fn wire_v3(reply: &Reply) -> Vec<u8> {
        wire_complete_version(reply, 3)
    }
    pub(crate) fn wire_complete_version(reply: &Reply, version: u16) -> Vec<u8> {
        let mut out = wire_base(reply, version);
        if let Some(f) = &reply.frame {
            out.extend([f.completion.is_some() as u8, 0, 0, 0]);
            if let Some(c) = &f.completion {
                let e = &c.event;
                out.extend(e.result_id);
                out.extend(e.match_id.to_be_bytes());
                out.extend(e.generation.to_be_bytes());
                out.extend(e.engine_revision.to_be_bytes());
                out.extend(e.finished_at_millis.to_be_bytes());
                out.extend([e.mode, c.state as u8, 0, 0]);
                out.extend(e.map_id.to_be_bytes());
                out.extend((e.participants.len() as u16).to_be_bytes());
                out.extend([0, 0]);
                for p in &e.participants {
                    if version >= 7 {
                        out.extend([p.kind as u8, 0, 0, 0]);
                    } else {
                        assert_eq!(p.kind, MemberKind::Human);
                    }
                    out.extend(p.entity_id.to_be_bytes());
                    out.extend(p.key.source_id.to_be_bytes());
                    out.extend(p.key.object_id.to_be_bytes());
                    out.extend([p.team, p.verdict as u8, 0, 0]);
                }
            }
            if version >= 4 {
                out.extend((f.casts.len() as u16).to_be_bytes());
                out.extend([0; 2]);
                for c in &f.casts {
                    out.extend(c.entity_id.to_be_bytes());
                    out.extend(c.request_sequence.to_be_bytes());
                    out.extend(c.cast_count.to_be_bytes());
                    out.extend(c.cast_revision.to_be_bytes());
                    out.extend(c.mp_spent.to_be_bytes());
                    out.extend(c.skill_id.to_be_bytes());
                    out.extend(c.from.x.to_be_bytes());
                    out.extend(c.from.y.to_be_bytes());
                    out.extend(c.to.x.to_be_bytes());
                    out.extend(c.to.y.to_be_bytes());
                    out.extend(c.cast_interval_millis.to_be_bytes());
                }
            }
            if version >= 15 {
                out.extend([f.descriptor.is_some() as u8, 0, 0, 0, 0, 0, 0, 0]);
                if let Some(descriptor) = &f.descriptor {
                    descriptor.encode(&mut out);
                }
            } else {
                assert!(f.descriptor.is_none());
            }
            if version >= 16 {
                super::super::race_protocol::encode(&mut out, f.race.as_ref());
            } else {
                assert!(f.race.is_none());
            }
            if version >= 20 {
                out.extend([f.audience_lane.map_or(0, |lane| lane + 1), 0, 0, 0]);
            } else {
                assert!(f.audience_lane.is_none());
            }
            if version >= 21 {
                super::super::quest_protocol::encode(&mut out, f.quest.as_ref());
            } else {
                assert!(f.quest.is_none());
            }
            if version >= 25 {
                assert!(
                    f.time_attack.is_none(),
                    "use Java oracle for cooperative state"
                );
                out.extend([0; 8]);
            }
            if version >= 26 {
                assert!(f.siege.is_none(), "use Java oracle for siege state");
                out.extend([0; 8]);
            }
        }
        let len = out.len() as u32;
        out[4..8].copy_from_slice(&len.to_be_bytes());
        out
    }
    #[test]
    fn version_three_completion_literal_appendix_and_old_shapes_stay_strict() {
        let reply = finished_reply();
        let bytes = wire_v3(&reply);
        let tail = hex(COMPLETION_TAIL);
        assert_eq!(tail.len(), 112);
        assert_eq!(&bytes[bytes.len() - tail.len()..], tail);
        assert_eq!(decode_body_version(&bytes[16..], 3).unwrap(), reply);
        for version in [1, 2] {
            assert!(app::decode_response_version(&bytes, version).is_err());
        }
        assert!(decode_body_version(&wire(&reply)[16..], 3).is_err());
        let old = sample();
        let old_bytes = wire(&old);
        let new_bytes = wire_v3(&old);
        assert_eq!(new_bytes.len(), old_bytes.len() + 4);
        for version in [1, 2] {
            assert_eq!(decode_body_version(&old_bytes[16..], version).unwrap(), old);
        }
        assert_eq!(decode_body_version(&new_bytes[16..], 3).unwrap(), old);
        assert!(decode_body_version(&old_bytes[16..], 3).is_err());
        assert!(decode_body_version(&new_bytes[16..], 2).is_err());
    }
    #[test]
    fn completion_is_bounded_exact_and_malformed_outcome_never_becomes_victory() {
        let good = finished_reply();
        let bytes = wire_v3(&good);
        let start = bytes.len() - 112;
        for n in 16..bytes.len() {
            assert!(
                decode_body_version(&bytes[16..n], 3).is_err(),
                "truncated at {n}"
            );
        }
        for offset in [
            start,
            start + 1,
            start + 53,
            start + 54,
            start + 62,
            start + 64 + 23,
            start + 88 + 21,
        ] {
            let mut b = bytes.clone();
            b[offset] = 255;
            assert!(decode_body_version(&b[16..], 3).is_err());
        }
        for case in 0..14 {
            let mut bad = good.clone();
            let f = bad.frame.as_mut().unwrap();
            let c = f.completion.as_mut().unwrap();
            match case {
                0 => c.event.result_id = [0; 16],
                1 => c.event.match_id = 10,
                2 => c.event.generation = 7,
                3 => c.event.engine_revision = 8,
                4 => c.event.engine_revision = -1,
                5 => c.event.finished_at_millis = -1,
                6 => c.event.mode = 2,
                7 => c.event.map_id = 504,
                8 => c.event.participants[0].key.source_id = 42,
                9 => c.event.participants.swap(0, 1),
                10 => c.event.participants[0].team = 1,
                11 => c.event.participants[1].verdict = Verdict::Win,
                12 => f.phase = Phase::Active,
                _ => {
                    c.event.participants.pop();
                }
            }
            assert!(bad.validate().is_err(), "case {case}");
            assert!(
                decode_body_version(&wire_v3(&bad)[16..], 3).is_err(),
                "wire case {case}"
            );
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(decode_body_version(&extra[16..], 3).is_err());
    }
    #[test]
    fn completion_receipt_is_monotonic_and_does_not_reopen_world_or_mutate_result() {
        let pending = finished_reply();
        let mut active = pending.clone();
        {
            let f = active.frame.as_mut().unwrap();
            f.phase = Phase::Active;
            f.revision = 7;
            f.completion = None;
        }
        assert!(follows(Some(&active), &pending));
        assert!(follows(Some(&pending), &pending));
        for state in [SettlementState::Saved, SettlementState::Failed] {
            let mut saved = pending.clone();
            let f = saved.frame.as_mut().unwrap();
            f.revision = 9;
            f.completion.as_mut().unwrap().state = state;
            assert!(follows(Some(&pending), &saved));
            assert!(follows(Some(&saved), &saved));
            assert!(!follows(Some(&saved), &pending));
            for case in 0..5 {
                let mut bad = saved.clone();
                let f = bad.frame.as_mut().unwrap();
                match case {
                    0 => f.revision = 10,
                    1 => f.completion.as_mut().unwrap().event.finished_at_millis += 1,
                    2 => f.actors[0].heading = 6,
                    3 => f.completion.as_mut().unwrap().state = SettlementState::Pending,
                    _ => {
                        f.phase = Phase::Aborted;
                        f.abort_reason = AbortReason::ServerAbort;
                        f.completion = None;
                    }
                }
                assert!(!follows(Some(&pending), &bad), "case {case}");
            }
        }
        let mut left = pending.clone();
        left.echo.action = Action::Leave;
        left.echo.match_id = 9;
        left.echo.generation = 6;
        left.frame = None;
        assert!(follows(Some(&pending), &left));
        let mut abort = active.clone();
        let f = abort.frame.as_mut().unwrap();
        f.phase = Phase::Aborted;
        f.abort_reason = AbortReason::ServerAbort;
        assert!(!follows(Some(&abort), &pending));
    }
    fn wire(reply: &Reply) -> Vec<u8> {
        wire_base(reply, 1)
    }
    fn wire_base(reply: &Reply, version: u16) -> Vec<u8> {
        fn text(out: &mut Vec<u8>, v: &str) {
            out.extend((v.len() as u16).to_be_bytes());
            out.extend(v.as_bytes());
        }
        fn key(out: &mut Vec<u8>, v: CharacterKey) {
            out.extend(v.source_id.to_be_bytes());
            out.extend(v.object_id.to_be_bytes());
        }
        let mut out = app::header(0xfedcba98, RESULT_KIND, 124).unwrap();
        put_request(&mut out, &reply.echo);
        out.extend([
            reply.status as u8,
            reply.reason as u8,
            reply.frame.is_some() as u8,
            0,
        ]);
        if let Some(f) = &reply.frame {
            out.extend(f.match_id.to_be_bytes());
            out.extend(f.generation.to_be_bytes());
            out.extend(f.revision.to_be_bytes());
            out.extend([f.phase as u8, f.abort_reason as u8, 0, 0]);
            let s = &f.room.summary;
            out.extend(s.room_id.to_be_bytes());
            out.extend(s.revision.to_be_bytes());
            out.extend(s.settings.capacity.to_be_bytes());
            out.extend([s.settings.mode, s.settings.attributes]);
            out.extend(s.settings.map_id.to_be_bytes());
            text(&mut out, &s.settings.name);
            out.extend(s.users.to_be_bytes());
            out.extend(s.ready.to_be_bytes());
            out.extend([s.satisfied as u8, 0, 0, 0]);
            key(&mut out, f.room.owner);
            out.extend((f.room.members.len() as u16).to_be_bytes());
            out.extend([0, 0]);
            for m in &f.room.members {
                if version >= 7 {
                    out.extend([m.kind as u8, 0, 0, 0]);
                } else {
                    assert_eq!(m.kind, MemberKind::Human);
                }
                key(&mut out, m.key);
                out.extend(m.level.to_be_bytes());
                out.extend(m.class.to_be_bytes());
                out.extend([m.sex, m.team, m.ready as u8, 0, 0, 0]);
                text(&mut out, &m.name);
                text(&mut out, &m.source_name);
            }
            let m = &f.manifest;
            out.extend(m.version.to_be_bytes());
            out.extend((m.sectors.len() as u16).to_be_bytes());
            out.extend(m.map_id.to_be_bytes());
            out.extend(m.digest);
            for s in &m.sectors {
                out.extend(s.x.to_be_bytes());
                out.extend(s.y.to_be_bytes());
                out.extend(s.byte_length.to_be_bytes());
                out.extend(s.sha256);
            }
            out.extend((f.actors.len() as u16).to_be_bytes());
            out.extend([0, 0]);
            for a in &f.actors {
                out.extend(a.entity_id.to_be_bytes());
                out.extend(a.position.x.to_be_bytes());
                out.extend(a.position.y.to_be_bytes());
                out.extend([a.heading, a.loaded as u8, 0, 0]);
                out.extend(a.sequence.to_be_bytes());
                out.extend(a.step_interval_nanos.to_be_bytes());
            }
        }
        let len = out.len() as u32;
        out[4..8].copy_from_slice(&len.to_be_bytes());
        out
    }
    #[test]
    fn independent_java_rust_full_frame_golden_and_exact_u64_epoch_i64_level() {
        let request = Request::read(context(), 3);
        assert_eq!(
            encode_request(0xfedcba98, &request).unwrap(),
            hex(REQUEST_HEX)
        );
        let decoded = app::decode_response(&hex(FRAME_HEX)).unwrap();
        assert_eq!(decoded.request_id, 0xfedcba98);
        let app::Response::World(reply) = decoded.response else {
            panic!("wrong response")
        };
        assert_eq!(reply.echo, request);
        let f = reply.frame.as_ref().unwrap();
        assert_eq!(f.room.members[0].level, 4_000_000_000);
        assert_eq!(f.manifest.map_id, 514);
        assert_eq!(f.actors[0].position, WorldCell { x: 32724, y: 32784 });
        assert_eq!(f.actors[0].step_interval_nanos, 50_000_000);
        assert_eq!(wire(&reply), hex(FRAME_HEX));
    }
    #[test]
    fn v7_world_and_complete_result_keep_bot_kind_and_do_not_match_human_self() {
        let mut reply = finished_reply();
        let f = reply.frame.as_mut().unwrap();
        let human = f.actors[0].key;
        f.room.members[1].kind = MemberKind::ServerBot;
        f.room.members[1].key = human;
        f.actors[1].kind = MemberKind::ServerBot;
        f.actors[1].key = human;
        let participant = &mut f.completion.as_mut().unwrap().event.participants[1];
        participant.kind = MemberKind::ServerBot;
        participant.key = human;
        f.validate(human).unwrap();
        assert!(f.actors[0].matches_character(human));
        assert!(!f.actors[1].matches_character(human));
        assert!(!f.completion.as_ref().unwrap().event.participants[1].matches_character(human));
        let bytes = wire_complete_version(&reply, 7);
        assert_eq!(decode_body_version(&bytes[16..], 7).unwrap(), reply);
        assert!(decode_body_version(&bytes[16..], 6).is_err());
        // v7 participant is kind/pad4 + original entity/key/team/verdict24.
        let last_participant = bytes.len() - 4 - 28;
        assert_eq!(
            &bytes[last_participant..last_participant + 4],
            &[1, 0, 0, 0]
        );
        for (p, v) in [
            (last_participant, 2),
            (last_participant + 1, 1),
            (last_participant + 2, 1),
            (last_participant + 3, 1),
        ] {
            let mut bad = bytes.clone();
            bad[p] = v;
            assert!(decode_body_version(&bad[16..], 7).is_err());
        }
        let mut wrong = reply.clone();
        wrong
            .frame
            .as_mut()
            .unwrap()
            .completion
            .as_mut()
            .unwrap()
            .event
            .participants[1]
            .kind = MemberKind::Human;
        assert!(wrong.validate().is_err());
        let mut wrong = reply.clone();
        wrong.frame.as_mut().unwrap().actors[1].kind = MemberKind::Human;
        assert!(wrong.validate().is_err());
        for end in 0..bytes.len() {
            assert!(app::decode_response_version(&bytes[..end], 7).is_err());
        }
    }
    #[test]
    fn every_truncation_extra_byte_bad_enum_and_reserved_rejected() {
        let good = hex(FRAME_HEX);
        for n in 0..good.len() {
            assert!(app::decode_response(&good[..n]).is_err(), "{n}");
        }
        for (offset, value) in [
            (39, 1),
            (37, 5),
            (120, 4),
            (121, 13),
            (122, 2),
            (123, 1),
            (148, 5),
            (149, 8),
            (150, 1),
        ] {
            let mut bad = good.clone();
            bad[offset] = value;
            assert!(app::decode_response(&bad).is_err(), "offset {offset}");
        }
        let mut bad = good;
        bad.push(0);
        let len = bad.len() as u32;
        bad[4..8].copy_from_slice(&len.to_be_bytes());
        assert!(app::decode_response(&bad).is_err());
    }
    #[test]
    fn request_unused_fields_and_signed_ranges_do_not_alias() {
        for i in 0..8 {
            let mut r = Request::read(context(), 3);
            match i {
                0 => r.heading = 1,
                1 => r.room_id = 1,
                2 => r.expected_room_revision = 1,
                3 => r.match_id = 1,
                4 => r.sequence = 1,
                5 => r.from_x = 1,
                6 => r.manifest_digest = [1; 32],
                _ => r.context.epoch = 0,
            }
            assert!(encode_request(1, &r).is_err(), "{i}");
        }
        let mut r = Request::read(context(), 3);
        r.action = Action::Move;
        r.match_id = i64::MAX as u64;
        r.generation = i64::MAX as u64;
        r.sequence = i64::MAX as u64;
        r.from_x = i32::MIN;
        r.from_y = i32::MAX;
        r.heading = 7;
        assert!(r.validate().is_ok());
        r.sequence += 1;
        assert!(r.validate().is_err());
        r.action = Action::Loaded;
        r.sequence = 0;
        r.heading = 0;
        r.from_x = 0;
        r.from_y = 0;
        assert!(r.validate().is_err());
        r.manifest_digest = [1; 32];
        assert!(r.validate().is_ok());
    }
    #[test]
    fn frame_cannot_claim_loaded_active_duplicate_position_or_wrong_identity() {
        let good = sample();
        for case in 0..11 {
            let mut bad = good.clone();
            let f = bad.frame.as_mut().unwrap();
            match case {
                0 => f.phase = Phase::Active,
                1 => f.phase = Phase::Aborted,
                2 => f.generation = 7,
                3 => f.actors[0].entity_id = 2,
                4 => f.actors[0].position.x = i32::MAX,
                5 => f.actors[0].step_interval_nanos = 0,
                6 => f.actors[0].sequence = u64::MAX,
                7 => f.room.members[0].key.source_id = 42,
                8 => f.room.summary.ready = 0,
                9 => f.manifest.version = 2,
                _ => bad.echo.context.key.source_id = 42,
            }
            assert!(bad.validate().is_err(), "{case}");
            assert!(app::decode_response(&wire(&bad)).is_err(), "wire {case}");
        }
    }
    #[test]
    fn source_scoped_same_object_members_are_distinct_and_ordered() {
        let mut value = sample();
        let f = value.frame.as_mut().unwrap();
        let mut member = f.room.members[0].clone();
        member.key.source_id = 42;
        f.room.members.push(member.clone());
        f.room.summary.users = 2;
        f.room.summary.ready = 2;
        let mut actor = f.actors[0].clone();
        actor.entity_id = 2;
        actor.key = member.key;
        actor.position.x += 1;
        f.actors.push(actor);
        value.validate().unwrap();
        assert!(app::decode_response(&wire(&value)).is_ok());
        let f = value.frame.as_mut().unwrap();
        f.actors[1].position = f.actors[0].position;
        assert!(value.validate().is_err());
    }
    #[test]
    fn manifest_count_sort_hash_per_file_and_total_byte_budgets() {
        for case in 0..6 {
            let mut bad = sample();
            let m = &mut bad.frame.as_mut().unwrap().manifest;
            match case {
                0 => m.sectors.clear(),
                1 => m.sectors.push(m.sectors[0].clone()),
                2 => m.sectors[0].byte_length = 8 * 1024 * 1024 + 1,
                3 => m.digest = [0; 32],
                4 => m.sectors[0].sha256 = [0; 32],
                _ => {
                    let original = m.sectors[0].clone();
                    m.sectors = (0..5)
                        .map(|i| {
                            let mut s = original.clone();
                            s.y += i;
                            s.byte_length = 8 * 1024 * 1024;
                            s
                        })
                        .collect();
                }
            }
            assert!(bad.validate().is_err());
            assert!(app::decode_response(&wire(&bad)).is_err());
        }
    }
    #[test]
    fn acknowledgements_confirm_loaded_move_and_leave_instead_of_optimistic_state() {
        let mut r = sample();
        r.echo.action = Action::Loaded;
        r.echo.match_id = 9;
        r.echo.generation = 6;
        r.echo.manifest_digest = [0x22; 32];
        assert!(r.validate().is_err());
        r.frame.as_mut().unwrap().actors[0].loaded = true;
        r.validate().unwrap();
        r.echo.action = Action::Move;
        r.echo.manifest_digest = [0; 32];
        r.echo.heading = 2;
        r.echo.sequence = 1;
        r.echo.from_x = 32724;
        r.echo.from_y = 32784;
        let f = r.frame.as_mut().unwrap();
        f.phase = Phase::Active;
        f.actors[0].position.x += 1;
        f.actors[0].sequence = 1;
        r.validate().unwrap();
        r.echo.sequence = 2;
        assert!(r.validate().is_err());
        r.echo = Request::read(context(), 3);
        r.echo.action = Action::Leave;
        r.echo.match_id = 9;
        r.echo.generation = 6;
        assert!(r.validate().is_err());
        r.frame = None;
        r.validate().unwrap();
        assert!(app::decode_response(&wire(&r)).is_ok());
    }
    #[test]
    fn versions_phase_identity_and_same_sequence_position_must_not_regress() {
        let old = sample();
        assert!(follows(Some(&old), &old));
        for case in 0..8 {
            let mut next = old.clone();
            let f = next.frame.as_mut().unwrap();
            f.revision += 1;
            match case {
                0 => f.match_id += 1,
                1 => f.generation += 1,
                2 => f.manifest.digest = [3; 32],
                3 => f.phase = Phase::Preparing,
                4 => f.actors[0].position.x += 1,
                5 => f.actors[0].step_interval_nanos += 1,
                6 => next.frame = None,
                _ => next.echo.context.epoch -= 1,
            }
            assert!(!follows(Some(&old), &next), "{case}");
        }
        let mut leave = old.clone();
        leave.echo.action = Action::Leave;
        leave.echo.match_id = 9;
        leave.echo.generation = 6;
        leave.frame = None;
        assert!(follows(Some(&old), &leave));
        leave.status = Status::Denied;
        leave.reason = Reason::InvalidState;
        assert!(!follows(Some(&old), &leave));
    }
    #[test]
    fn preparing_to_loading_preserves_frozen_room_but_can_qualify_interval() {
        let mut old = sample();
        old.frame.as_mut().unwrap().phase = Phase::Preparing;
        let mut next = old.clone();
        let f = next.frame.as_mut().unwrap();
        f.phase = Phase::Loading;
        f.revision += 1;
        f.actors[0].step_interval_nanos += 1;
        assert!(follows(Some(&old), &next));
        next.frame.as_mut().unwrap().room.members[0].level += 1;
        assert!(!follows(Some(&old), &next));
    }
    #[test]
    fn max_members_text_sector_payload_is_bounded_and_only_kind21_owns_extra_limit() {
        let mut value = sample();
        let f = value.frame.as_mut().unwrap();
        f.room.summary.settings.capacity = 100;
        f.room.summary.users = 100;
        f.room.summary.ready = 100;
        let m = f.room.members[0].clone();
        let a = f.actors[0].clone();
        f.room.members = (0..100)
            .map(|i| {
                let mut row = m.clone();
                row.key.object_id += i;
                row.name = "n".repeat(256);
                row.source_name = "s".repeat(256);
                row
            })
            .collect();
        f.actors = f
            .room
            .members
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let mut row = a.clone();
                row.key = m.key;
                row.entity_id = i as u64 + 1;
                row.position = WorldCell {
                    x: 32704 + (i % 64) as i32,
                    y: 32768 + (i / 64) as i32,
                };
                row
            })
            .collect();
        value.validate().unwrap();
        let wire = wire(&value);
        assert!(wire.len() > 50_000 && wire.len() < MAX_FRAME_BYTES);
        assert!(app::decode_response(&wire).is_ok());
        for kind in [20_u16, 21] {
            let limit = if kind == 21 {
                MAX_FRAME_BYTES
            } else {
                app::MAX_FRAME_BYTES
            };
            let mut header = hex(FRAME_HEX)[..16].to_vec();
            header[12..14].copy_from_slice(&kind.to_be_bytes());
            header[4..8].copy_from_slice(&((limit + 1) as u32).to_be_bytes());
            assert!(app::decode_response(&header).is_err());
        }
    }
}
