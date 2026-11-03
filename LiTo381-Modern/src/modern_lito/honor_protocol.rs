//! v13 NEW_MATCH_HONOR_V1. Objectives are not source characters or BOTs.
//! Every value is authority state; presentation never resurrects from a timer.
use super::{
    application_protocol::{self as app, ApplicationError},
    combat_protocol as combat,
};

pub const MODE: u8 = 5;
pub const OBJECTIVE_BYTES: usize = 96;
pub const LIFE_BYTES: usize = 64;
pub const HEAL_BYTES: usize = 88;
pub const BASIC_HEAL: u32 = 1;
pub const HEAL_EFFECT: i32 = 11722;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Participant(u64),
    Objective(u64),
}
impl Target {
    pub fn handle(self) -> u64 {
        match self {
            Self::Participant(id) | Self::Objective(id) => id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Interaction {
    None,
    Physical,
    LightArrow,
    Heal,
    FireStorm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objective {
    pub entity_id: u64,
    pub team: u8,
    pub action: u8,
    pub graphics_id: i32,
    pub position: super::world_map::WorldCell,
    pub hp: i64,
    pub max_hp: i64,
    /// Last accepted interaction, including a successful zero-restoration heal.
    /// This is NOT an instruction to restart an unchanged tower action.
    pub state_revision: i64,
    pub hurt_revision: i64,
    pub death_revision: i64,
    pub damage_loss: i64,
    pub heal_gain: i64,
    pub last_actor_entity_id: u64,
    pub last_kind: Interaction,
}
impl Objective {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Life {
    pub entity_id: u64,
    pub death_count: i64,
    pub last_death_revision: i64,
    pub respawn_count: i64,
    pub respawn_revision: i64,
    pub respawn_hp_gain: i64,
    /// ACTIVE-relative due time, not local wall time. Zero while alive/terminal.
    pub pending_due_millis: i64,
    pub death_position: super::world_map::WorldCell,
}
impl Life {
    pub fn initial(entity_id: u64) -> Self {
        Self {
            entity_id,
            death_count: 0,
            last_death_revision: 0,
            respawn_count: 0,
            respawn_revision: 0,
            respawn_hp_gain: 0,
            pending_due_millis: 0,
            death_position: super::world_map::WorldCell { x: 0, y: 0 },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealCast {
    pub entity_id: u64,
    pub count: i64,
    pub revision: i64,
    pub mp_spent: i64,
    pub target_objective_id: u64,
    pub hp_before: i64,
    pub hp_after: i64,
    pub restored: i64,
    pub from: super::world_map::WorldCell,
    pub to: super::world_map::WorldCell,
    pub skill_id: u32,
    /// Frozen fresh-source effective MP cost. Zero is qualified, not missing.
    pub mp_cost: i32,
}
impl HealCast {
    pub fn initial(entity_id: u64, mp_cost: i32) -> Self {
        Self {
            entity_id,
            count: 0,
            revision: 0,
            mp_spent: 0,
            target_objective_id: 0,
            hp_before: 0,
            hp_after: 0,
            restored: 0,
            from: super::world_map::WorldCell { x: 0, y: 0 },
            to: super::world_map::WorldCell { x: 0, y: 0 },
            skill_id: 0,
            mp_cost,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub objectives: Vec<Objective>,
    pub lives: Vec<Life>,
    pub heals: Vec<HealCast>,
}
impl State {
    pub fn objective(&self, id: u64) -> Option<&Objective> {
        self.objectives.iter().find(|v| v.entity_id == id)
    }
    pub fn life(&self, id: u64) -> Option<&Life> {
        self.lives.iter().find(|v| v.entity_id == id)
    }
    pub fn heal(&self, id: u64) -> Option<&HealCast> {
        self.heals.iter().find(|v| v.entity_id == id)
    }
}

pub fn target(frame: &combat::Frame, id: u64) -> Option<Target> {
    if frame.world.actors.iter().any(|v| v.entity_id == id) {
        Some(Target::Participant(id))
    } else if frame
        .honor
        .as_ref()
        .is_some_and(|h| h.objective(id).is_some())
    {
        Some(Target::Objective(id))
    } else {
        None
    }
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid authoritative Honor state",
    )
}
pub fn target_hp(frame: &combat::Frame, id: u64) -> Option<i64> {
    frame
        .vital(id)
        .map(|v| v.hp)
        .or_else(|| frame.honor.as_ref()?.objective(id).map(|o| o.hp))
}
pub fn target_position(frame: &combat::Frame, id: u64) -> Option<super::world_map::WorldCell> {
    frame
        .world
        .actors
        .iter()
        .find(|a| a.entity_id == id)
        .map(|a| a.position)
        .or_else(|| frame.honor.as_ref()?.objective(id).map(|o| o.position))
}
pub fn target_team(frame: &combat::Frame, id: u64) -> Option<u8> {
    frame
        .world
        .actors
        .iter()
        .position(|a| a.entity_id == id)
        .map(|i| frame.world.room.members[i].team)
        .or_else(|| frame.honor.as_ref()?.objective(id).map(|o| o.team))
}
pub fn target_hurt_revision(frame: &combat::Frame, id: u64) -> Option<i64> {
    frame
        .vital(id)
        .map(|v| v.hurt_revision)
        .or_else(|| frame.honor.as_ref()?.objective(id).map(|o| o.hurt_revision))
}

pub fn action_for_hp(hp: i64, max_hp: i64) -> u8 {
    if hp == 0 {
        35
    } else if hp < max_hp / 4 {
        34
    } else if hp < max_hp / 2 {
        33
    } else if i128::from(hp) < i128::from(max_hp) * 3 / 4 {
        32
    } else {
        0
    }
}

pub fn in_screen(from: super::world_map::WorldCell, to: super::world_map::WorldCell) -> bool {
    let dx = i64::from(to.x) - i64::from(from.x);
    let dy = i64::from(to.y) - i64::from(from.y);
    let distance = dx.abs() + dy.abs();
    distance <= 18 || distance == 19 && (dx + 18).abs() + (dy + 18).abs() <= 52
}
pub fn respawn_gain(f: &combat::Frame, entity: u64) -> i64 {
    f.honor
        .as_ref()
        .and_then(|h| h.life(entity))
        .map_or(0, |l| l.respawn_hp_gain)
}
pub fn respawn_revision(f: &combat::Frame, entity: u64) -> i64 {
    f.honor
        .as_ref()
        .and_then(|h| h.life(entity))
        .map_or(0, |l| l.respawn_revision)
}
pub fn respawned(a: &combat::Frame, b: &combat::Frame, entity: u64) -> bool {
    a.honor
        .as_ref()
        .and_then(|h| h.life(entity))
        .zip(b.honor.as_ref().and_then(|h| h.life(entity)))
        .is_some_and(|(x, y)| y.respawn_count > x.respawn_count)
}
pub fn heal_spent(f: &combat::Frame, entity: u64) -> i64 {
    f.honor
        .as_ref()
        .and_then(|h| h.heal(entity))
        .map_or(0, |h| h.mp_spent)
}
pub fn heal_revision(f: &combat::Frame, entity: u64) -> i64 {
    f.honor
        .as_ref()
        .and_then(|h| h.heal(entity))
        .map_or(0, |h| h.revision)
}

impl Objective {
    fn validate(&self) -> Result<(), ApplicationError> {
        if self.entity_id < 3
            || self.entity_id > i64::MAX as u64
            || self.team > 1
            || ![2665, 2667, 2669, 2671].contains(&self.graphics_id)
            || self.max_hp < 1
            || self.hp < 0
            || self.hp > self.max_hp
            || self.state_revision < 0
            || self.hurt_revision < 0
            || self.hurt_revision > self.state_revision
            || self.death_revision < 0
            || self.death_revision > self.state_revision
            || self.damage_loss < 0
            || self.heal_gain < 0
            || self.action != action_for_hp(self.hp, self.max_hp)
            || (self.hp == 0) != (self.death_revision > 0)
            || self.death_revision > 0 && self.death_revision != self.hurt_revision
            || (self.state_revision == 0) != (self.last_kind == Interaction::None)
            || (self.state_revision == 0) != (self.last_actor_entity_id == 0)
            || self.state_revision == 0
                && (self.hp != self.max_hp
                    || self.hurt_revision != 0
                    || self.damage_loss != 0
                    || self.heal_gain != 0)
            || i128::from(self.max_hp) + i128::from(self.heal_gain) - i128::from(self.damage_loss)
                != i128::from(self.hp)
        {
            return Err(invalid());
        }
        Ok(())
    }
}
impl Life {
    fn validate(&self) -> Result<(), ApplicationError> {
        if self.entity_id == 0
            || self.entity_id > i64::MAX as u64
            || self.death_count < 0
            || self.respawn_count < 0
            || self.respawn_count > self.death_count
            || self.death_count - self.respawn_count > 1
            || self.last_death_revision < 0
            || self.respawn_revision < 0
            || self.respawn_hp_gain < 0
            || !(0..=1_805_000).contains(&self.pending_due_millis)
            || (self.death_count == 0) != (self.last_death_revision == 0)
            || (self.respawn_count == 0) != (self.respawn_revision == 0)
            || (self.respawn_count == 0) != (self.respawn_hp_gain == 0)
            || self.death_count == 0
                && (self.death_position.x != 0
                    || self.death_position.y != 0
                    || self.pending_due_millis != 0)
            || self.death_count == self.respawn_count
                && (self.pending_due_millis != 0
                    || self.death_count > 0 && self.respawn_revision <= self.last_death_revision)
            || self.death_count > self.respawn_count
                && self.last_death_revision <= self.respawn_revision
        {
            return Err(invalid());
        }
        Ok(())
    }
}
impl HealCast {
    fn validate(&self) -> Result<(), ApplicationError> {
        if self.entity_id == 0
            || self.entity_id > i64::MAX as u64
            || self.count < 0
            || self.revision < 0
            || self.mp_spent < 0
            || self.hp_before < 0
            || self.hp_after < self.hp_before
            || self.restored < 0
            || self.hp_after - self.hp_before != self.restored
            || !(0..=4).contains(&self.mp_cost)
            || self.count.checked_mul(i64::from(self.mp_cost)) != Some(self.mp_spent)
            || if self.count == 0 {
                *self != Self::initial(self.entity_id, self.mp_cost)
            } else {
                self.revision == 0
                    || self.target_objective_id < 3
                    || self.hp_before == 0
                    || self.skill_id != BASIC_HEAL
                    || !in_screen(self.from, self.to)
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
}
pub fn validate_frame(f: &combat::Frame) -> Result<(), ApplicationError> {
    use super::world_protocol::Phase;
    let Some(h) = &f.honor else { return Ok(()) };
    let n = f.world.actors.len();
    if !(2..=100).contains(&n)
        || h.objectives.len() != 2
        || h.lives.len() != n
        || h.heals.len() != n
        || f.world.room.summary.settings.mode != MODE
        || f.world.casts.len() != n
        || f.regen.len() != n
        || h.objectives[0].position == h.objectives[1].position
    {
        return Err(invalid());
    }
    let initial = matches!(f.world.phase, Phase::Loading | Phase::Countdown);
    for (index, o) in h.objectives.iter().enumerate() {
        o.validate()?;
        if o.entity_id != (n + index + 1) as u64
            || o.team != index as u8
            || o.last_actor_entity_id > n as u64
            || !f.world.manifest.contains(o.position)
            || o.state_revision > f.world.revision
            || f.world.actors.iter().any(|a| a.position == o.position)
            || initial && o.state_revision != 0
        {
            return Err(invalid());
        }
        if o.last_actor_entity_id > 0 {
            let actor = &f.world.actors[o.last_actor_entity_id as usize - 1];
            if (o.last_kind == Interaction::Heal)
                != (f.world.room.members[actor.entity_id as usize - 1].team == o.team)
            {
                return Err(invalid());
            }
            if o.state_revision == f.world.revision
                && match o.last_kind {
                    Interaction::Physical => f.vital(actor.entity_id).is_none_or(|v| {
                        v.attack_revision != o.state_revision
                            || v.last_target_entity_id != o.entity_id
                    }),
                    Interaction::LightArrow => f.damage(actor.entity_id).is_none_or(|d| {
                        d.cast_revision != o.state_revision as u64
                            || d.target_entity_id != o.entity_id
                    }),
                    Interaction::Heal => h.heal(actor.entity_id).is_none_or(|v| {
                        v.revision != o.state_revision || v.target_objective_id != o.entity_id
                    }),
                    Interaction::FireStorm => f.fire(actor.entity_id).is_none_or(|s| {
                        s.cast_revision != o.state_revision
                            || !s
                                .targets
                                .iter()
                                .any(|t| t.target_entity_id == o.entity_id && t.applied_damage > 0)
                    }),
                    Interaction::None => true,
                }
            {
                return Err(invalid());
            }
        }
    }
    for (i, ((l, hc), a)) in h
        .lives
        .iter()
        .zip(&h.heals)
        .zip(&f.world.actors)
        .enumerate()
    {
        l.validate()?;
        hc.validate()?;
        let v = f.vitals.get(i).ok_or_else(invalid)?;
        let cast = &f.world.casts[i];
        if l.entity_id != a.entity_id
            || hc.entity_id != a.entity_id
            || l.last_death_revision != v.death_revision
            || l.last_death_revision > f.world.revision
            || l.respawn_revision > f.world.revision
            || (v.hp == 0) != (l.death_count > l.respawn_count)
            || v.hp == 0 && l.last_death_revision != v.hurt_revision
            || l.respawn_count.checked_mul(v.max_hp) != Some(l.respawn_hp_gain)
            || initial && (l.death_count != 0 || hc.count != 0)
            || f.world.phase != Phase::Active && l.pending_due_millis != 0
            || f.world.phase == Phase::Active && v.hp == 0 && l.pending_due_millis == 0
            || l.death_count > 0 && !f.world.manifest.contains(l.death_position)
            || l.respawn_revision == f.world.revision
                && (v.hp != v.max_hp || a.position != l.death_position)
            || hc.revision > f.world.revision
            || i128::from(hc.count)
                + i128::from(cast.cast_count)
                + i128::from(f.damage(a.entity_id).map_or(0, |d| d.cast_count))
                > i128::from(cast.request_sequence)
            || a.identity().character().is_none() && *hc != HealCast::initial(a.entity_id, 0)
        {
            return Err(invalid());
        }
        if hc.count > 0 {
            let o = h.objective(hc.target_objective_id).ok_or_else(invalid)?;
            if o.team != f.world.room.members[i].team
                || hc.to != o.position
                || hc.hp_after > o.max_hp
                || !f.world.manifest.contains(hc.from)
                || o.state_revision < hc.revision
                || o.heal_gain < hc.restored
                || hc.revision == f.world.revision
                    && (o.state_revision != hc.revision
                        || o.last_kind != Interaction::Heal
                        || o.hp != hc.hp_after
                        || o.last_actor_entity_id != a.entity_id
                        || a.position != hc.from)
            {
                return Err(invalid());
            }
        }
    }
    if let Some(c) = &f.world.completion {
        let end = c.event.engine_revision.checked_add(1).ok_or_else(invalid)?;
        let dead: Vec<_> = h.objectives.iter().filter(|o| o.hp == 0).collect();
        if c.event.mode != MODE
            || dead.len() != 1
            || dead[0].death_revision != end
            || c.event.participants.iter().any(|p| {
                (p.verdict == super::world_protocol::Verdict::Loss) != (p.team == dead[0].team)
            })
            || h.objectives.iter().any(|o| o.state_revision > end)
            || h.lives
                .iter()
                .any(|l| l.last_death_revision > end || l.respawn_revision > end)
            || h.heals.iter().any(|h| h.revision > end)
        {
            return Err(invalid());
        }
    } else if h.objectives.iter().any(|o| o.hp == 0) {
        return Err(invalid());
    }
    Ok(())
}

pub fn follows(a: &combat::Frame, b: &combat::Frame) -> bool {
    let (Some(x), Some(y)) = (&a.honor, &b.honor) else {
        return a.honor.is_none() && b.honor.is_none();
    };
    if x.objectives.len() != y.objectives.len()
        || x.lives.len() != y.lives.len()
        || x.heals.len() != y.heals.len()
    {
        return false;
    }
    if matches!(
        a.world.phase,
        super::world_protocol::Phase::Finished | super::world_protocol::Phase::Aborted
    ) {
        return x == y;
    }
    for (old, new) in x.objectives.iter().zip(&y.objectives) {
        if old.entity_id != new.entity_id
            || old.team != new.team
            || old.graphics_id != new.graphics_id
            || old.position != new.position
            || old.max_hp != new.max_hp
            || new.state_revision < old.state_revision
            || new.hurt_revision < old.hurt_revision
            || new.death_revision < old.death_revision
            || new.damage_loss < old.damage_loss
            || new.heal_gain < old.heal_gain
            || old.hp == 0 && old != new
            || old.state_revision == new.state_revision && old != new
            || old.state_revision != new.state_revision && new.state_revision <= a.world.revision
            || (new.damage_loss > old.damage_loss) != (new.hurt_revision > old.hurt_revision)
            || new.hurt_revision > old.hurt_revision && new.hurt_revision <= a.world.revision
        {
            return false;
        }
        // Every tower healing gain is backed by a member's new spell history.
        let mut heal_bound = 0i128;
        let mut heal_minimum = 0i128;
        for (ah, bh) in x.heals.iter().zip(&y.heals) {
            let count = bh.count - ah.count;
            if count > 0 {
                heal_bound += i128::from(count - 1) * i128::from(new.max_hp);
                if bh.target_objective_id == new.entity_id {
                    heal_bound += i128::from(bh.restored);
                    heal_minimum += i128::from(bh.restored);
                }
            }
        }
        if !(heal_minimum..=heal_bound).contains(&i128::from(new.heal_gain - old.heal_gain)) {
            return false;
        }
    }
    for (i, (old, new)) in x.lives.iter().zip(&y.lives).enumerate() {
        if old.entity_id != new.entity_id
            || new.death_count < old.death_count
            || new.respawn_count < old.respawn_count
            || new.last_death_revision < old.last_death_revision
            || new.respawn_revision < old.respawn_revision
            || new.respawn_hp_gain < old.respawn_hp_gain
            || (new.death_count == old.death_count)
                != (new.last_death_revision == old.last_death_revision)
            || (new.respawn_count == old.respawn_count)
                != (new.respawn_revision == old.respawn_revision)
            || new.death_count > old.death_count && new.last_death_revision <= a.world.revision
            || new.respawn_count > old.respawn_count && new.respawn_revision <= a.world.revision
            || old.death_count == new.death_count && old.death_position != new.death_position
            || old.death_count == new.death_count
                && old.respawn_count == new.respawn_count
                && old.pending_due_millis != new.pending_due_millis
                && !(matches!(
                    b.world.phase,
                    super::world_protocol::Phase::Finished | super::world_protocol::Phase::Aborted
                ) && new.pending_due_millis == 0)
            || old.death_count > old.respawn_count
                && new.respawn_count == old.respawn_count
                && a.world.actors[i].position != b.world.actors[i].position
        {
            return false;
        }
    }
    for (old, new) in x.heals.iter().zip(&y.heals) {
        if old.entity_id != new.entity_id
            || old.mp_cost != new.mp_cost
            || new.count < old.count
            || new.revision < old.revision
            || new.mp_spent < old.mp_spent
            || old.count == new.count && old != new
            || new.count > old.count && new.revision <= a.world.revision
        {
            return false;
        }
        let (Some(ac), Some(bc)) = (a.world.cast(old.entity_id), b.world.cast(new.entity_id))
        else {
            return false;
        };
        let total = i128::from(new.count - old.count) + i128::from(bc.cast_count)
            - i128::from(ac.cast_count)
            + i128::from(b.damage(new.entity_id).map_or(0, |d| d.cast_count))
            - i128::from(a.damage(old.entity_id).map_or(0, |d| d.cast_count));
        if total > i128::from(bc.request_sequence) - i128::from(ac.request_sequence) {
            return false;
        }
    }
    true
}

pub(crate) fn negotiated(response: &app::Response, enabled: bool) -> bool {
    let f = match response {
        app::Response::Combat(r) => r.frame.as_ref(),
        app::Response::Siege(r) => r.frame.as_ref(),
        app::Response::Cast(r) => r.frame.as_ref(),
        app::Response::Item(r) => r.frame.as_ref(),
        app::Response::Ammo(r) => r.frame.as_ref(),
        _ => return true,
    };
    // A capable listener can retain non-Honor mappings. Mode 5 is the opt-in.
    f.is_none_or(|f| {
        if f.world.descriptor.is_some() {
            f.world.feature_presence_matches(
                super::world_protocol::MatchDescriptor::HONOR,
                f.honor.is_some(),
                enabled,
            )
        } else if f.world.room.summary.settings.mode == MODE {
            f.honor.is_some() == enabled
        } else {
            f.honor.is_none()
        }
    })
}

pub(crate) fn decode(
    bytes: &[u8],
    actors: usize,
) -> Result<(Option<State>, usize), ApplicationError> {
    struct R<'a> {
        bytes: &'a [u8],
        pos: usize,
    }
    impl<'a> R<'a> {
        fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
            let end = self.pos.checked_add(n).ok_or_else(invalid)?;
            let s = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
            self.pos = end;
            Ok(s)
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
        fn zero(&mut self, n: usize) -> Result<(), ApplicationError> {
            if self.take(n)?.iter().any(|b| *b != 0) {
                Err(invalid())
            } else {
                Ok(())
            }
        }
        fn cell(&mut self) -> Result<super::world_map::WorldCell, ApplicationError> {
            Ok(super::world_map::WorldCell {
                x: self.u32()? as i32,
                y: self.u32()? as i32,
            })
        }
    }
    let mut r = R { bytes, pos: 0 };
    let enabled = r.u8()?;
    let oc = usize::from(r.u8()?);
    let lc = usize::from(r.u16()?);
    let hc = usize::from(r.u16()?);
    r.zero(2)?;
    if enabled > 1
        || enabled == 0 && (oc != 0 || lc != 0 || hc != 0)
        || enabled == 1 && (!(2..=100).contains(&actors) || oc != 2 || lc != actors || hc != actors)
        || bytes.len() - r.pos < oc * OBJECTIVE_BYTES + lc * LIFE_BYTES + hc * HEAL_BYTES
    {
        return Err(invalid());
    }
    if enabled == 0 {
        return Ok((None, r.pos));
    }
    let mut objectives = Vec::with_capacity(oc);
    let mut lives = Vec::with_capacity(lc);
    let mut heals = Vec::with_capacity(hc);
    for _ in 0..oc {
        let entity_id = r.u64()?;
        let team = r.u8()?;
        let action = r.u8()?;
        r.zero(2)?;
        let graphics_id = r.u32()? as i32;
        let position = r.cell()?;
        let hp = r.i64()?;
        let max_hp = r.i64()?;
        let state_revision = r.i64()?;
        let hurt_revision = r.i64()?;
        let death_revision = r.i64()?;
        let damage_loss = r.i64()?;
        let heal_gain = r.i64()?;
        let last_actor_entity_id = r.u64()?;
        let last_kind = match r.u8()? {
            0 => Interaction::None,
            1 => Interaction::Physical,
            2 => Interaction::LightArrow,
            3 => Interaction::Heal,
            4 => Interaction::FireStorm,
            _ => return Err(invalid()),
        };
        r.zero(7)?;
        let o = Objective {
            entity_id,
            team,
            action,
            graphics_id,
            position,
            hp,
            max_hp,
            state_revision,
            hurt_revision,
            death_revision,
            damage_loss,
            heal_gain,
            last_actor_entity_id,
            last_kind,
        };
        o.validate()?;
        objectives.push(o);
    }
    for _ in 0..lc {
        let l = Life {
            entity_id: r.u64()?,
            death_count: r.i64()?,
            last_death_revision: r.i64()?,
            respawn_count: r.i64()?,
            respawn_revision: r.i64()?,
            respawn_hp_gain: r.i64()?,
            pending_due_millis: r.i64()?,
            death_position: r.cell()?,
        };
        l.validate()?;
        lives.push(l);
    }
    for _ in 0..hc {
        let h = HealCast {
            entity_id: r.u64()?,
            count: r.i64()?,
            revision: r.i64()?,
            mp_spent: r.i64()?,
            target_objective_id: r.u64()?,
            hp_before: r.i64()?,
            hp_after: r.i64()?,
            restored: r.i64()?,
            from: r.cell()?,
            to: r.cell()?,
            skill_id: r.u32()?,
            mp_cost: r.u32()? as i32,
        };
        h.validate()?;
        heals.push(h);
    }
    Ok((
        Some(State {
            objectives,
            lives,
            heals,
        }),
        r.pos,
    ))
}

#[cfg(test)]
pub(crate) fn encode(out: &mut Vec<u8>, state: Option<&State>) {
    let Some(s) = state else {
        out.extend([0; 8]);
        return;
    };
    out.extend([1, s.objectives.len() as u8]);
    out.extend((s.lives.len() as u16).to_be_bytes());
    out.extend((s.heals.len() as u16).to_be_bytes());
    out.extend([0; 2]);
    for o in &s.objectives {
        out.extend(o.entity_id.to_be_bytes());
        out.extend([o.team, o.action, 0, 0]);
        out.extend(o.graphics_id.to_be_bytes());
        out.extend(o.position.x.to_be_bytes());
        out.extend(o.position.y.to_be_bytes());
        for v in [
            o.hp,
            o.max_hp,
            o.state_revision,
            o.hurt_revision,
            o.death_revision,
            o.damage_loss,
            o.heal_gain,
        ] {
            out.extend(v.to_be_bytes());
        }
        out.extend(o.last_actor_entity_id.to_be_bytes());
        out.push(o.last_kind as u8);
        out.extend([0; 7]);
    }
    for l in &s.lives {
        out.extend(l.entity_id.to_be_bytes());
        for v in [
            l.death_count,
            l.last_death_revision,
            l.respawn_count,
            l.respawn_revision,
            l.respawn_hp_gain,
            l.pending_due_millis,
        ] {
            out.extend(v.to_be_bytes());
        }
        out.extend(l.death_position.x.to_be_bytes());
        out.extend(l.death_position.y.to_be_bytes());
    }
    for h in &s.heals {
        out.extend(h.entity_id.to_be_bytes());
        for v in [h.count, h.revision, h.mp_spent] {
            out.extend(v.to_be_bytes());
        }
        out.extend(h.target_objective_id.to_be_bytes());
        for v in [h.hp_before, h.hp_after, h.restored] {
            out.extend(v.to_be_bytes());
        }
        for v in [h.from.x, h.from.y, h.to.x, h.to.y] {
            out.extend(v.to_be_bytes());
        }
        out.extend(h.skill_id.to_be_bytes());
        out.extend(h.mp_cost.to_be_bytes());
    }
}
#[cfg(test)]
#[path = "honor_protocol_tests.rs"]
pub(crate) mod tests;
