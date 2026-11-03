//! Identity/visibility checks for typed candidate data. This is deliberately
//! NOT the final gameplay proof or admission type: event arithmetic, temporal
//! following, acknowledgements and durable receipt validation remain separate.
use super::{data::*, invalid, ApplicationError};
use crate::modern_lito::{application_protocol as app, participant, rooms, world_protocol as old};

fn checked<T: TryFrom<i32>>(n: i32) -> Result<T, ApplicationError> {
    T::try_from(n).map_err(|_| invalid())
}
fn positive(n: i64) -> Result<u64, ApplicationError> {
    if n > 0 {
        Ok(n as u64)
    } else {
        Err(invalid())
    }
}
fn key(source: i32, object: i64) -> Result<app::CharacterKey, ApplicationError> {
    let k = app::CharacterKey {
        source_id: checked(source)?,
        object_id: positive(object)?,
    };
    if !app::valid_source(k.source_id) {
        return Err(invalid());
    }
    Ok(k)
}
fn identity(i: &Identity) -> Result<participant::ParticipantId, ApplicationError> {
    Ok(participant::ParticipantId::new(
        match i.kind {
            MemberKind::Human => participant::MemberKind::Human,
            MemberKind::ServerBot => participant::MemberKind::ServerBot,
        },
        key(i.source, i.id)?,
    ))
}
impl WorldId {
    fn valid(&self) -> bool {
        self.room > 0 && self.match_id > 0 && self.generation > 0
    }
}
impl Player {
    fn valid(&self) -> bool {
        // Epoch is an opaque nonzero 64-bit token, NOT a positive signed ID.
        self.source > 0 && self.object > 0 && self.epoch != 0 && (0..=3).contains(&self.channel)
    }
}
impl UnvalidatedData {
    /// Check against the caller's independently selected session/room context.
    /// Ok does NOT grant permission to publish this data: the type stays
    /// UnvalidatedData, and no conversion to legacy Frames is performed.
    pub fn check_context(
        &self,
        expected_player: &Player,
        expected_world: &WorldId,
    ) -> Result<(), ApplicationError> {
        if self.vip.is_some() {
            super::vip::attached(self)?.check_binding(expected_player, expected_world)?;
            return self.check_vip_context(expected_player, expected_world);
        }
        if self.armor.is_some() {
            super::armor::attached(self)?;
            return self.check_armor_context(expected_player, expected_world);
        }
        if self.healing.is_some() {
            super::healing::attached(self)?;
            return self.check_healing_context(expected_player, expected_world);
        }
        if self.speed.is_some() {
            super::speed::attached(self)?;
            return self.check_context_kind(expected_player, expected_world, false, true, false);
        }
        if self.haste.is_some() {
            super::haste::attached(self)?;
            self.check_context_kind(expected_player, expected_world, true, false, false)
        } else {
            self.check_context_kind(expected_player, expected_world, false, false, false)
        }
    }
    pub(super) fn check_haste_context(
        &self,
        expected_player: &Player,
        expected_world: &WorldId,
    ) -> Result<(), ApplicationError> {
        self.check_context_kind(expected_player, expected_world, true, false, false)
    }
    pub(super) fn check_speed_context(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        if self.haste.is_some() || self.npc_motion.is_some() {
            return Err(invalid());
        }
        self.check_context_kind(player, world, false, true, false)
    }
    pub(super) fn check_healing_context(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        let speed = self.world.descriptor.features & super::speed::FEATURE != 0;
        let haste = !speed && self.world.descriptor.features & super::haste::FEATURE != 0;
        self.check_context_kind(player, world, haste, speed, true)
    }
    fn check_context_kind(
        &self,
        expected_player: &Player,
        expected_world: &WorldId,
        haste_candidate: bool,
        speed_candidate: bool,
        healing_candidate: bool,
    ) -> Result<(), ApplicationError> {
        self.check_context_extensions(
            expected_player,
            expected_world,
            haste_candidate,
            speed_candidate,
            healing_candidate,
            false,
            false,
        )
    }
    pub(super) fn check_armor_context(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        let f = self.world.descriptor.features;
        let speed = f & super::speed::FEATURE != 0;
        let haste = !speed && f & super::haste::FEATURE != 0;
        self.check_context_extensions(
            player,
            world,
            haste,
            speed,
            f & super::healing::FEATURE != 0,
            true,
            false,
        )
    }
    pub(super) fn check_vip_context(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        let f = self.world.descriptor.features;
        let speed = f & super::speed::FEATURE != 0;
        let haste = !speed && f & super::haste::FEATURE != 0;
        self.check_context_extensions(
            player,
            world,
            haste,
            speed,
            f & super::healing::FEATURE != 0,
            f & super::armor::FEATURE != 0,
            true,
        )
    }
    fn check_context_extensions(
        &self,
        expected_player: &Player,
        expected_world: &WorldId,
        haste_candidate: bool,
        speed_candidate: bool,
        healing_candidate: bool,
        armor_candidate: bool,
        vip_candidate: bool,
    ) -> Result<(), ApplicationError> {
        let w = &self.world;
        let p = &self.personal;
        let room = &self.room;
        // Schema6 is admitted only to its explicit candidate checker, not to
        // live or legacy entry points. Manually constructed values must not
        // lose reagent authority through an older check.
        if (w.descriptor.features & super::vip::FEATURE != 0) != vip_candidate
            || (w.descriptor.features & super::armor::FEATURE != 0) != armor_candidate
            || (w.descriptor.features & super::healing::FEATURE != 0) != healing_candidate
            || p.history.as_ref().is_some_and(|h| h.0.healing.is_some()) != healing_candidate
            || p.materials.is_some() != speed_candidate
            || !expected_player.valid()
            || !expected_world.valid()
            || p.player != *expected_player
            || w.identity != *expected_world
            || w.revision < 1
            || w.identity.room != room.id
            || w.identity.generation != room.revision
            || !(1..=100).contains(&p.entity)
            || w.loading_elapsed_nanos < 0
            || w.active_elapsed_nanos < -1
            || w.active_elapsed_nanos > w.loading_elapsed_nanos
            || (w.phase == Phase::Aborted) != (w.abort != Abort::None)
        {
            return Err(invalid());
        }
        let player_key = key(p.player.source, p.player.object)?;
        let members = room
            .members
            .0
            .iter()
            .map(|m| {
                Ok(rooms::Member {
                    kind: identity(&m.identity)?.kind,
                    key: key(m.identity.source, m.identity.id)?,
                    level: m.level,
                    class: checked(m.class)?,
                    sex: checked(m.sex)?,
                    team: checked(m.team)?,
                    ready: m.ready,
                    name: m.name.0.clone(),
                    source_name: m.source_name.0.clone(),
                })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        let roster = rooms::RoomSnapshot {
            summary: rooms::Summary {
                room_id: positive(room.id)?,
                revision: room.revision,
                settings: rooms::PublicDraft {
                    capacity: checked(room.capacity)?,
                    mode: checked(room.mode)?,
                    attributes: checked(room.attributes)?,
                    map_id: checked(room.map)?,
                    name: room.name.0.clone(),
                },
                users: checked(room.users)?,
                ready: checked(room.ready)?,
                satisfied: room.satisfied,
            },
            owner: key(room.owner_source, room.owner_object)?,
            members,
        };
        roster.validate_members(player_key)?;
        let desc = &w.descriptor;
        let descriptor = old::MatchDescriptor {
            ruleset: match desc.ruleset {
                Ruleset::SceneOnly => old::MatchRuleset::SceneOnly,
                Ruleset::PvpSkirmish => old::MatchRuleset::PvpSkirmish,
                Ruleset::DeathmatchElimination => old::MatchRuleset::DeathmatchElimination,
                Ruleset::HonorTowers => old::MatchRuleset::HonorTowers,
                Ruleset::PetRace => old::MatchRuleset::PetRace,
                Ruleset::NpcEncounter => old::MatchRuleset::NpcEncounter,
                Ruleset::CooperativeTimeAttack => old::MatchRuleset::CooperativeTimeAttack,
                Ruleset::SiegeObjectives => old::MatchRuleset::SiegeObjectives,
            },
            features: checked(desc.features)?,
            preparation_timeout_millis: checked(desc.preparation_timeout_millis)?,
            load_timeout_millis: checked(desc.load_timeout_millis)?,
            countdown_nanos: u64::try_from(desc.countdown_nanos).map_err(|_| invalid())?,
            active_timeout_millis: checked(desc.active_timeout_millis)?,
        };
        if vip_candidate {
            descriptor.validate_vip_candidate(roster.summary.settings.mode)?;
        } else if armor_candidate {
            descriptor.validate_armor_candidate(roster.summary.settings.mode)?;
        } else if healing_candidate {
            descriptor.validate_healing_candidate(roster.summary.settings.mode)?;
        } else if speed_candidate {
            descriptor.validate_speed_candidate(roster.summary.settings.mode)?;
        } else if haste_candidate {
            descriptor.validate_haste_candidate(roster.summary.settings.mode)?;
        } else {
            descriptor.validate_canonical(roster.summary.settings.mode)?;
        }
        if descriptor.has(old::MatchDescriptor::HONOR) != !self.honor.objectives.0.is_empty()
            || descriptor.has(old::MatchDescriptor::NPC) != self.npcs.is_some()
            || descriptor.has(old::MatchDescriptor::TIME_ATTACK) != self.time_attack.is_some()
            || descriptor.has(old::MatchDescriptor::SIEGE) != self.siege.is_some()
            || descriptor.has(old::MatchDescriptor::PET_RACE) != self.race.is_some()
            || self.quest.is_some() && room.mode != 2
        {
            return Err(invalid());
        }
        let manifest = old::Manifest {
            version: 1,
            map_id: checked(self.manifest.map)?,
            digest: self.manifest.digest,
            sectors: self
                .manifest
                .sectors
                .0
                .iter()
                .map(|s| {
                    Ok(old::Sector {
                        x: checked(s.x)?,
                        y: checked(s.y)?,
                        byte_length: checked(s.byte_length)?,
                        sha256: s.sha256,
                    })
                })
                .collect::<Result<Vec<_>, ApplicationError>>()?,
        };
        manifest.validate()?;
        let isolated = descriptor.has(old::MatchDescriptor::ISOLATED_FORCES);
        if !(-1..=1).contains(&w.lane) || isolated != (w.lane >= 0) {
            return Err(invalid());
        }
        let expected = roster
            .members
            .iter()
            .enumerate()
            .filter(|(_, m)| w.lane < 0 || i32::from(m.team) == w.lane)
            .collect::<Vec<_>>();
        if expected.len() != self.actors.0.len() || expected.is_empty() {
            return Err(invalid());
        }
        for (a, (index, m)) in self.actors.0.iter().zip(expected) {
            if a.entity != index as i64 + 1
                || !(0..=7).contains(&a.heading)
                || a.last_move_sequence < 0
                || !(1_000_000..=10_000_000_000).contains(&a.step_interval_nanos)
                || !manifest.contains(crate::modern_lito::world_map::WorldCell {
                    x: a.position.x,
                    y: a.position.y,
                })
                || matches!(w.phase, Phase::Countdown | Phase::Active | Phase::Finished)
                    && !a.loaded
                || descriptor.combat() != a.vital.is_some()
                || descriptor.combat() != a.resources.is_some()
                || descriptor.has(old::MatchDescriptor::TIMED) != a.buffs.is_some()
                || descriptor.has(old::MatchDescriptor::REGEN) != a.regeneration.is_some()
                || a.entity == p.entity && !m.matches_character(player_key)
            {
                return Err(invalid());
            }
            for s in &a.spells.histories.0 {
                if let Some(r) = &s.receipt {
                    for e in &r.impact.targets.0 {
                        self.target_context(&e.target)?;
                    }
                }
            }
            if let Some(s) = &a.spells.structure {
                for e in &s.impact.targets.0 {
                    self.target_context(&e.target)?;
                }
            }
        }
        let owner = self
            .actors
            .0
            .iter()
            .find(|a| a.entity == p.entity)
            .ok_or_else(invalid)?;
        let member = roster
            .members
            .get(usize::try_from(p.entity - 1).map_err(|_| invalid())?)
            .ok_or_else(invalid)?;
        if !member.matches_character(player_key)
            || descriptor.combat() != p.equipment.is_some()
            || descriptor.combat() != p.history.is_some()
            || descriptor.has(old::MatchDescriptor::ITEMS) != p.supplies.is_some()
            || descriptor.has(old::MatchDescriptor::WEIGHT) != p.weight.is_some()
            || descriptor.has(old::MatchDescriptor::AMMO) != p.ammo.is_some()
            || descriptor.has(old::MatchDescriptor::REGEN) != p.regen_clock.is_some()
        {
            return Err(invalid());
        }
        if let Some(e) = &p.equipment {
            if e.source != p.player.source
                || e.object != p.player.object
                || e.name.0 != member.name
                || e.class != i32::from(member.class)
                || e.sex != i32::from(member.sex)
                || e.level != member.level
            {
                return Err(invalid());
            }
        }
        if let Some(h) = &p.history {
            let h = &h.0;
            let id = h.identity;
            if id.world.room_id != w.identity.room as u64
                || id.world.match_id != w.identity.match_id as u64
                || id.world.generation != w.identity.generation as u64
                || id.source.key != player_key
                || id.source.epoch != u64::from_be_bytes(p.player.epoch.to_be_bytes())
                || i32::from(id.channel_id) != p.player.channel
                || id.actor.entity_id != p.entity as u64
                || id.actor.identity != member.identity()
                || h.world_revision != w.revision
                || owner.resources.as_ref().is_none_or(|r| {
                    r.revision != h.resources.revision
                        || r.equipment_revision != h.resources.equipment_revision
                })
                || p.equipment
                    .as_ref()
                    .is_none_or(|e| e.revision != h.resources.equipment_revision)
            {
                return Err(invalid());
            }
        }
        if let Some(n) = &self.npcs {
            if n.actors.0.iter().any(|a| w.lane >= 0 && a.lane != w.lane) {
                return Err(invalid());
            }
            for e in &n.exchanges.0 {
                if e.scope != w.identity || !self.visible_identity(&e.participant) {
                    return Err(invalid());
                }
            }
            let mut names = std::collections::BTreeSet::new();
            let mut graphics = std::collections::BTreeSet::new();
            for name in &n.names.0 {
                if name.source <= 0 || name.npc_id <= 0 || !names.insert((name.source, name.npc_id))
                {
                    return Err(invalid());
                }
                graphics.insert(name.graphic);
            }
            if graphics.into_iter().collect::<Vec<_>>() != n.graphics.0
                || n.actors.0.iter().any(|a| {
                    !n.names.0.iter().any(|n| {
                        n.source == a.source && n.npc_id == a.npc_id && n.graphic == a.graphic
                    })
                })
            {
                return Err(invalid());
            }
        }
        if let Some(q) = &self.quest {
            if q.lanes.0.len() != if isolated { 1 } else { 2 }
                || q.lanes.0.iter().any(|l| w.lane >= 0 && l.lane != w.lane)
                || q.targets.0.iter().any(|t| w.lane >= 0 && t.lane != w.lane)
                || q.lives
                    .0
                    .iter()
                    .any(|l| w.lane >= 0 && l.lane != w.lane || !self.visible_entity(l.entity))
            {
                return Err(invalid());
            }
            for o in &q.objectives.0 {
                self.quest_context(&o.id)?;
                if w.lane >= 0 && o.target.lane != w.lane
                    || i64::from(o.id.slot)
                        != i64::from(o.target.lane) * 4 + i64::from(o.target.stage)
                {
                    return Err(invalid());
                }
            }
            for e in &q.exchanges.0 {
                self.quest_context(&e.target)?;
                if !self.visible_identity(&e.participant) {
                    return Err(invalid());
                }
            }
        }
        if let Some(t) = &self.time_attack {
            if t.exchanges
                .0
                .iter()
                .any(|e| e.target != w.identity || !self.visible_entity(e.entity))
            {
                return Err(invalid());
            }
        }
        if self.siege.as_ref().is_some_and(|s| s.scope != w.identity) {
            return Err(invalid());
        }
        if let Some(r) = &self.race {
            if r.runners.0.len() != self.actors.0.len()
                || r.runners
                    .0
                    .iter()
                    .zip(&self.actors.0)
                    .any(|(r, a)| r.entity != a.entity)
            {
                return Err(invalid());
            }
        }
        if let Some(r) = &self.result {
            if r.match_id != w.identity.match_id
                || r.generation != w.identity.generation
                || r.engine_revision < 1
                || r.engine_revision > w.revision
                || r.mode != room.mode
                || r.map != room.map
                || r.participants.0.len() != room.members.0.len()
            {
                return Err(invalid());
            }
            for (index, (p, m)) in r.participants.0.iter().zip(&room.members.0).enumerate() {
                if p.entity != index as i64 + 1 || p.identity != m.identity || p.team != m.team {
                    return Err(invalid());
                }
            }
            if r.time_attack.as_ref().is_some_and(|p| {
                p.room != w.identity.room
                    || p.physical_map != self.manifest.map
                    || p.terrain_sha256 != self.manifest.digest
            }) || r.siege.as_ref().is_some_and(|p| {
                p.room != w.identity.room
                    || p.physical_map != self.manifest.map
                    || p.terrain_sha256 != self.manifest.digest
            }) {
                return Err(invalid());
            }
        }
        Ok(())
    }
    fn visible_entity(&self, id: i64) -> bool {
        self.actors.0.iter().any(|a| a.entity == id)
    }
    fn visible_identity(&self, id: &Identity) -> bool {
        self.actors.0.iter().any(|a| {
            usize::try_from(a.entity - 1)
                .ok()
                .and_then(|n| self.room.members.0.get(n))
                .is_some_and(|m| m.identity == *id)
        })
    }
    fn quest_context(&self, id: &QuestId) -> Result<(), ApplicationError> {
        if id.scope != self.world.identity
            || !(1..=8).contains(&id.slot)
            || self.world.lane >= 0 && (id.slot - 1) / 4 != self.world.lane
        {
            Err(invalid())
        } else {
            Ok(())
        }
    }
    fn target_context(&self, t: &Target) -> Result<(), ApplicationError> {
        match t {
            Target::Participant { entity, identity } => {
                let m = usize::try_from(entity.checked_sub(1).ok_or_else(invalid)?)
                    .ok()
                    .and_then(|n| self.room.members.0.get(n))
                    .ok_or_else(invalid)?;
                if m.identity != *identity || !self.visible_entity(*entity) {
                    return Err(invalid());
                }
            }
            Target::Npc { scope, .. } | Target::TimeTower(scope) => {
                if *scope != self.world.identity {
                    return Err(invalid());
                }
            }
            Target::Quest { id, .. } => self.quest_context(id)?,
            Target::Honor { entity, team } => {
                if !self
                    .honor
                    .objectives
                    .0
                    .iter()
                    .any(|o| o.entity == *entity && o.team == *team)
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
}
