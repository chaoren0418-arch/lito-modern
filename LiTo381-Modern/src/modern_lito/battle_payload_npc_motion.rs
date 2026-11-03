//! Schema4, mandatory for explicitly negotiated v43 BATTLE_DATA.
//! The first thirteen sections use schema 2 (or 3 for Greater Haste); movement
//! is an additional complete, owner-filtered roster, never a command proof.
use super::{
    codec::{record, Reader, Wire, Writer},
    combat::require,
    data::*,
    invalid, ApplicationError, Section,
};

pub const SCHEMA: u16 = 4;
pub const VERSION: u16 = 43;
pub const SECTION: usize = 14;
pub const LIMIT: usize = 45 + 128 * 81;
pub const MAX_BYTES: usize = super::MAX_PRESENTATION_BYTES + 8 + LIMIT;
record!(Walk [56] {
    sequence: i64, life: i64, roster_revision: i64, elapsed_millis: i64,
    duration_millis: i32, from: Cell, to: Cell, heading: i32
});
record!(Actor [81] {
    serial: i64, life: i64, walk_millis: i32, attack_millis: i32,
    last_walk: Option<Walk>
});
record!(State [LIMIT - 1] {
    world: WorldId, revision: i64, elapsed_millis: i64, actors: List<Actor, 128>
});
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub state: Option<State>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub data: UnvalidatedData,
    // Explicit absence is only legal for a world without NPCs. Loading and
    // closed rosters are present states with zero resident actors.
    pub motion: Option<State>,
}

pub fn decode(bytes: &[u8]) -> Result<Candidate, ApplicationError> {
    let mut p = super::inspect_inner(bytes, true)?;
    require(p.schema() == SCHEMA)?;
    let mut r = Reader::new(p.section(Section::World));
    let world: World = r.read()?;
    r.finish()?;
    p.schema = if world.descriptor.features & (1 << 21) != 0 {
        super::GREATER_HASTE_SCHEMA
    } else {
        super::PRESENTATION_SCHEMA
    };
    let data = super::data::decode_payload(&p)?;
    let mut r = Reader::new(p.sections[SECTION - 1]);
    let motion = r.read()?;
    r.finish()?;
    Ok(Candidate { data, motion })
}

pub fn encode(candidate: &Candidate) -> Result<Vec<u8>, ApplicationError> {
    require(
        candidate.data.presentation.is_some()
            && candidate.data.npc_motion.is_none()
            && candidate.data.haste.is_none()
            && candidate.data.healing.is_none()
            && candidate.data.speed.is_none(),
    )?;
    let mut bytes = super::encode(&candidate.data)?;
    let mut count = Writer::count();
    count.write(&candidate.motion)?;
    require(count.pos <= LIMIT)?;
    let mut section = Writer::allocated(count.pos)?;
    section.write(&candidate.motion)?;
    let extra = 8 + count.pos;
    require(section.pos == count.pos && bytes.len() <= MAX_BYTES - extra)?;
    bytes.try_reserve_exact(extra).map_err(|_| invalid())?;
    bytes.extend((SECTION as u16).to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend((count.pos as u32).to_be_bytes());
    bytes.extend(section.bytes.ok_or_else(invalid)?);
    let n = bytes.len() as u32;
    bytes[4..6].copy_from_slice(&SCHEMA.to_be_bytes());
    bytes[8..12].copy_from_slice(&n.to_be_bytes());
    bytes[12..14].copy_from_slice(&(SECTION as u16).to_be_bytes());
    Ok(bytes)
}

impl Candidate {
    pub fn check_values(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        self.data.check_scene_values(player, world)?;
        self.check_motion()
    }
    fn check_motion(&self) -> Result<(), ApplicationError> {
        require(
            self.data.npc_motion.is_none()
                && self.data.haste.is_none()
                && self.data.healing.is_none()
                && self.data.speed.is_none(),
        )?;
        MotionView {
            data: &self.data,
            motion: &self.motion,
        }
        .check_motion()
    }
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        crate::modern_lito::battle_follow::check_candidate_sequence(&self.data, &next.data)?;
        self.check_motion()?;
        next.check_motion()?;
        MotionView {
            data: &self.data,
            motion: &self.motion,
        }
        .check_successor(&MotionView {
            data: &next.data,
            motion: &next.motion,
        })
    }
}
pub(super) fn check_attached(data: &UnvalidatedData) -> Result<(), ApplicationError> {
    if let Some(envelope) = &data.npc_motion {
        MotionView {
            data,
            motion: &envelope.state,
        }
        .check_motion()?;
    }
    Ok(())
}
pub(crate) fn follows_attached(
    a: &UnvalidatedData,
    b: &UnvalidatedData,
) -> Result<(), ApplicationError> {
    match (&a.npc_motion, &b.npc_motion) {
        (None, None) => Ok(()),
        (Some(x), Some(y)) => MotionView {
            data: a,
            motion: &x.state,
        }
        .check_successor(&MotionView {
            data: b,
            motion: &y.state,
        }),
        _ => Err(invalid()),
    }
}
pub(super) struct MotionView<'a> {
    pub data: &'a UnvalidatedData,
    pub motion: &'a Option<State>,
}
impl MotionView<'_> {
    pub(super) fn check_motion(&self) -> Result<(), ApplicationError> {
        require(self.data.presentation.is_some())?;
        let (Some(n), Some(m)) = (&self.data.npcs, &self.motion) else {
            return require(self.data.npcs.is_none() && self.motion.is_none());
        };
        require(
            m.world == self.data.world.identity
                && m.revision == n.revision
                && m.elapsed_millis == n.elapsed_millis
                && m.actors.0.len() == n.actors.0.len()
                && m.actors.0.len() <= 128,
        )?;
        for (a, v) in n.actors.0.iter().zip(&m.actors.0) {
            require(
                a.serial == v.serial
                    && a.life == v.life
                    && (1..=60_000).contains(&v.walk_millis)
                    && (1..=60_000).contains(&v.attack_millis),
            )?;
            let Some(w) = &v.last_walk else {
                require(a.action != NpcAction::Walk)?;
                continue;
            };
            require(
                w.sequence > 0
                    && w.life > 0
                    && w.life <= a.life
                    && w.roster_revision > a.born_revision
                    && w.roster_revision <= a.changed_revision
                    && w.sequence <= w.roster_revision - a.born_revision
                    && (0..=m.elapsed_millis).contains(&w.elapsed_millis)
                    && w.duration_millis == v.walk_millis
                    && w.to == a.cell
                    && self.data.manifest.contains_cell(&w.from)
                    && (0..=7).contains(&w.heading),
            )?;
            let (dx, dy) = [
                (0, -1),
                (1, -1),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
            ][w.heading as usize];
            require(
                i64::from(w.to.x) - i64::from(w.from.x) == dx
                    && i64::from(w.to.y) - i64::from(w.from.y) == dy,
            )?;
            if a.action == NpcAction::Walk {
                require(
                    w.life == a.life
                        && w.heading == a.heading
                        && a.next_move_millis == w.elapsed_millis + i64::from(w.duration_millis),
                )?;
            }
        }
        Ok(())
    }

    /// Candidate-only consistency check. Does not create an Observation or
    /// authorize gameplay/ACKs. Skipped observations retain only the last step.
    pub(super) fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        self.check_motion()?;
        next.check_motion()?;
        let (Some(a), Some(b)) = (&self.motion, &next.motion) else {
            return require(self.motion.is_none() && next.motion.is_none());
        };
        require(b.revision >= a.revision && b.elapsed_millis >= a.elapsed_millis)?;
        if matches!(self.data.world.phase, Phase::Aborted | Phase::Finished)
            || a.revision == b.revision
        {
            return require(a == b);
        }
        let old_n = self.data.npcs.as_ref().ok_or_else(invalid)?;
        for p in &a.actors.0 {
            let Some(q) = b.actors.0.iter().find(|q| q.serial == p.serial) else {
                continue;
            };
            require(p.walk_millis == q.walk_millis && p.attack_millis == q.attack_millis)?;
            let old_sequence = p.last_walk.as_ref().map_or(0, |w| w.sequence);
            let new_sequence = q.last_walk.as_ref().map_or(0, |w| w.sequence);
            require(
                new_sequence >= old_sequence
                    && new_sequence - old_sequence <= b.revision - a.revision,
            )?;
            if old_sequence == new_sequence {
                require(p.last_walk == q.last_walk)?;
                let old = old_n
                    .actors
                    .0
                    .iter()
                    .find(|n| n.serial == p.serial)
                    .ok_or_else(invalid)?;
                let new = next
                    .data
                    .npcs
                    .as_ref()
                    .and_then(|n| n.actors.0.iter().find(|n| n.serial == p.serial))
                    .ok_or_else(invalid)?;
                require(old.cell == new.cell)?;
            } else {
                let w = q.last_walk.as_ref().ok_or_else(invalid)?;
                require(
                    w.roster_revision > a.revision
                        && w.life >= p.life
                        && w.elapsed_millis >= a.elapsed_millis,
                )?;
                if let Some(prev) = &p.last_walk {
                    require(
                        w.elapsed_millis >= prev.elapsed_millis + i64::from(prev.duration_millis),
                    )?;
                }
                if new_sequence - old_sequence == 1 {
                    let old = old_n
                        .actors
                        .0
                        .iter()
                        .find(|n| n.serial == p.serial)
                        .ok_or_else(invalid)?;
                    require(w.from == old.cell)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "battle_payload_npc_motion_tests.rs"]
mod tests;
