//! Complete skill1 intent/receipt. Live use requires the explicit COMMANDS
//! extension and an authenticated owner. No client-supplied cost or clock.
use super::{
    application_protocol as app,
    battle_operation::Lifecycle,
    battle_payload::{data as d, healing},
    battle_window::Stamp,
    participant,
};
pub const REQUEST_KIND: u16 = 78;
pub const RESULT_KIND: u16 = 79;
pub const REQUEST_BYTES: usize = 160;
pub const RESULT_PREFIX_BYTES: usize = 192;
pub const MAX_RESULT_BYTES: usize = RESULT_PREFIX_BYTES + healing::MAX_BYTES;
#[path = "battle_player_healing_wire.rs"]
mod wire;
pub use wire::{
    decode_candidate_intent, decode_candidate_reply, encode_candidate_intent,
    encode_candidate_reply, read_candidate_reply,
};
pub(crate) use wire::{decode_reply_selected, encode_intent_selected, read_reply_selected};
pub(crate) fn negotiated(peer: super::protocol::extensions::Selected) -> bool {
    peer.allows_commands() && super::battle_world::negotiated(&peer.base())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None,
    NotActive,
    Dead,
    Unlearned,
    Unsupported,
    TooFast,
    NoMp,
    TooHeavy,
    StaleState,
    Numeric,
    TargetDead,
    StaleTarget,
    WrongInstance,
    OutOfRange,
    Blocked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub entity: i64,
    pub identity: participant::ParticipantId,
    pub life: i64,
    pub equipment_revision: i64,
    pub x: i32,
    pub y: i32,
}
impl Target {
    fn valid(&self) -> bool {
        (1..=100).contains(&self.entity)
            && self.identity.valid()
            && self.life > 0
            && self.equipment_revision >= 0
    }
    fn from_data(data: &d::UnvalidatedData, entity: i64) -> Option<Self> {
        let a = actor(data, entity).ok()?;
        let r = a.resources.as_ref()?;
        let m = data
            .room
            .members
            .0
            .get(usize::try_from(entity.checked_sub(1)?).ok()?)?;
        Some(Self {
            entity,
            identity: participant::ParticipantId {
                kind: match m.identity.kind {
                    d::MemberKind::Human => participant::MemberKind::Human,
                    d::MemberKind::ServerBot => participant::MemberKind::ServerBot,
                },
                source_id: m.identity.source.try_into().ok()?,
                id: m.identity.id.try_into().ok()?,
            },
            life: r.life.checked_add(1)?,
            equipment_revision: r.equipment_revision,
            x: a.position.x,
            y: a.position.y,
        })
    }
    fn matches(&self, e: &d::Effect) -> bool {
        matches!(&e.target,d::Target::Participant{entity,identity} if *entity==self.entity
            && identity.kind as u8==self.identity.kind as u8&&identity.source as u32==self.identity.source_id&&identity.id as u64==self.identity.id)
            && e.life == self.life
            && e.equipment_revision == self.equipment_revision
            && e.cell.x == self.x
            && e.cell.y == self.y
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Intent {
    pub request_id: u32,
    pub stamp: Stamp,
    pub sequence: i64,
    pub equipment_revision: i64,
    pub x: i32,
    pub y: i32,
    pub target: Target,
}
impl std::fmt::Debug for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PlayerHealingIntent[redacted]")
    }
}
impl Intent {
    pub fn from_candidate(
        checked: &healing::Checked,
        target: i64,
        request_id: u32,
    ) -> Result<Self, app::ApplicationError> {
        let c = checked.candidate();
        Self::from_view(&c.view()?, target, request_id)
    }
    pub(crate) fn from_observation(
        observation: &super::battle_follow::Observation,
        target: i64,
    ) -> Option<Self> {
        Self::from_view(&healing::attached(observation.data()).ok()?, target, 1).ok()
    }
    pub(crate) fn current(&self, observation: &super::battle_follow::Observation) -> bool {
        self.stamp.current(observation)
            && Self::from_observation(observation, self.target.entity).is_some_and(|q| {
                Self {
                    request_id: self.request_id,
                    stamp: self.stamp,
                    ..q
                } == *self
            })
    }
    fn from_view(
        c: &healing::View<'_>,
        target: i64,
        request_id: u32,
    ) -> Result<Self, app::ApplicationError> {
        let data = &c.data;
        let a = actor(data, data.personal.entity)?;
        let r = a.resources.as_ref().ok_or_else(invalid)?;
        let p = &data.personal.player;
        let w = &data.world.identity;
        require(data.world.phase == d::Phase::Active && a.loaded && r.vitals.hp > 0)?;
        let q = Self {
            request_id,
            stamp: Stamp {
                context: app::SelectionContext {
                    key: app::CharacterKey {
                        source_id: p.source as u32,
                        object_id: p.object as u64,
                    },
                    epoch: p.epoch as u64,
                },
                channel: p.channel.try_into().map_err(|_| invalid())?,
                world: [w.room, w.match_id, w.generation],
                entity: a.entity,
                life: r.life.checked_add(1).ok_or_else(invalid)?,
                revision: data.world.revision,
            },
            sequence: a.spells.sequence.checked_add(1).ok_or_else(invalid)?,
            equipment_revision: r.equipment_revision,
            x: a.position.x,
            y: a.position.y,
            target: Target::from_data(data, target).ok_or_else(invalid)?,
        };
        q.check_before_view(c)?;
        Ok(q)
    }
    pub fn validate(&self) -> Result<(), app::ApplicationError> {
        let s = self.stamp;
        require(
            self.request_id != 0
                && app::valid_context(s.context)
                && s.channel <= 3
                && s.world.iter().all(|n| *n > 0)
                && (1..=100).contains(&s.entity)
                && s.life > 0
                && s.revision > 0
                && self.sequence > 0
                && self.equipment_revision >= 0
                && self.target.valid(),
        )?;
        if self.target.entity == s.entity {
            require(
                self.target.identity.matches_character(s.context.key)
                    && self.target.life == s.life
                    && self.target.equipment_revision == self.equipment_revision
                    && self.target.x == self.x
                    && self.target.y == self.y,
            )?;
        }
        Ok(())
    }
    fn player(&self) -> d::Player {
        let s = self.stamp;
        d::Player {
            source: s.context.key.source_id as i32,
            object: s.context.key.object_id as i64,
            epoch: s.context.epoch as i64,
            channel: s.channel.into(),
        }
    }
    fn world(&self) -> d::WorldId {
        let s = self.stamp;
        d::WorldId {
            room: s.world[0],
            match_id: s.world[1],
            generation: s.world[2],
        }
    }
    pub fn check_before(&self, c: &healing::Candidate) -> Result<(), app::ApplicationError> {
        self.check_before_view(&c.view()?)
    }
    pub(crate) fn check_before_data(
        &self,
        data: &d::UnvalidatedData,
    ) -> Result<(), app::ApplicationError> {
        self.check_before_view(&healing::attached(data)?)
    }
    fn check_before_view(&self, c: &healing::View<'_>) -> Result<(), app::ApplicationError> {
        self.validate()?;
        c.check_values(&self.player(), &self.world())?;
        let d = &c.data;
        let a = actor(d, self.stamp.entity)?;
        let r = a.resources.as_ref().ok_or_else(invalid)?;
        require(
            d.personal.entity == self.stamp.entity
                && d.world.revision >= self.stamp.revision
                && r.life.checked_add(1) == Some(self.stamp.life)
                && r.equipment_revision == self.equipment_revision
                && a.position.x == self.x
                && a.position.y == self.y
                && a.spells.sequence.checked_add(1) == Some(self.sequence)
                && Target::from_data(d, self.target.entity) == Some(self.target),
        )
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct CandidateReply {
    pub echo: Intent,
    pub reason: Reason,
    pub lifecycle: Lifecycle,
    pub battle: healing::Candidate,
}
impl std::fmt::Debug for CandidateReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PlayerHealingReceipt[redacted]")
    }
}
impl CandidateReply {
    fn view(&self) -> Result<ReceiptView<'_>, app::ApplicationError> {
        Ok(ReceiptView {
            echo: self.echo,
            reason: self.reason,
            lifecycle: self.lifecycle,
            battle: self.battle.view()?,
        })
    }
    pub fn validate(&self) -> Result<(), app::ApplicationError> {
        self.view()?.validate()
    }
    pub fn check_after(
        &self,
        expected: &Intent,
        before: &healing::Candidate,
    ) -> Result<(), app::ApplicationError> {
        self.view()?.check_after(expected, &before.view()?)
    }
}
/// Full ordinary battle data; never converted back to a stripped candidate.
#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub echo: Intent,
    pub reason: Reason,
    pub lifecycle: Lifecycle,
    pub data: d::UnvalidatedData,
}
impl std::fmt::Debug for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PlayerHealingReceipt[redacted]")
    }
}
impl Reply {
    fn view(&self) -> Result<ReceiptView<'_>, app::ApplicationError> {
        Ok(ReceiptView {
            echo: self.echo,
            reason: self.reason,
            lifecycle: self.lifecycle,
            battle: healing::attached(&self.data)?,
        })
    }
    pub(crate) fn validate(&self) -> Result<(), app::ApplicationError> {
        self.view()?.validate()
    }
    pub(crate) fn check_after(
        &self,
        expected: &Intent,
        before: &d::UnvalidatedData,
    ) -> Result<(), app::ApplicationError> {
        self.view()?
            .check_after(expected, &healing::attached(before)?)
    }
}
struct ReceiptView<'a> {
    echo: Intent,
    reason: Reason,
    lifecycle: Lifecycle,
    battle: healing::View<'a>,
}
impl ReceiptView<'_> {
    fn validate(&self) -> Result<(), app::ApplicationError> {
        let q = self.echo;
        q.validate()?;
        self.battle.check_values(&q.player(), &q.world())?;
        let d = &self.battle.data;
        let w = &d.world;
        let a = actor(d, q.stamp.entity)?;
        self.lifecycle.validate(d)?;
        require(d.personal.entity == q.stamp.entity && q.stamp.revision <= w.revision)?;
        let r = outgoing(&self.battle, q.stamp.entity)?.receipt.as_ref();
        if self.reason == Reason::NotActive {
            return require(
                self.lifecycle.phase != super::world_protocol::Phase::Active
                    || w.phase != d::Phase::Active,
            );
        }
        require(
            w.phase == d::Phase::Active
                && a.spells.sequence == q.sequence
                && a.spells.request_revision == w.revision
                && w.revision > q.stamp.revision,
        )?;
        if self.reason == Reason::None {
            let i = &r.ok_or_else(invalid)?.impact;
            let u = &i.usage;
            require(
                u.sequence == q.sequence
                    && u.revision == w.revision
                    && u.before.equipment_revision == q.equipment_revision
                    && u.before.life == q.stamp.life
                    && u.before.cell.x == q.x
                    && u.before.cell.y == q.y
                    && q.target.matches(&i.target)
                    && entry(&self.battle, q.target.entity)?.incoming.last.as_ref() == Some(i),
            )?;
        } else {
            require(r.is_none_or(|r| r.impact.usage.sequence < q.sequence))?;
        }
        if self.reason == Reason::Dead {
            require(a.resources.as_ref().ok_or_else(invalid)?.vitals.hp == 0)?;
        }
        Ok(())
    }
    /// Exact pending echo and complete predecessor, not just a plausible current
    /// HP/MP. Other players may act while this command is awaiting its reply.
    pub fn check_after(
        &self,
        expected: &Intent,
        before: &healing::View<'_>,
    ) -> Result<(), app::ApplicationError> {
        require(self.echo == *expected)?;
        self.validate()?;
        expected.check_before_view(before)?;
        before.check_successor(&self.battle)?;
        let old = actor(&before.data, expected.stamp.entity)?;
        let new = actor(&self.battle.data, expected.stamp.entity)?;
        let a = outgoing(before, expected.stamp.entity)?;
        let b = outgoing(&self.battle, expected.stamp.entity)?;
        if self.reason == Reason::NotActive {
            require(
                new.spells.sequence == old.spells.sequence
                    && new.spells.request_revision == old.spells.request_revision
                    || new.spells.sequence == expected.sequence
                        && new.spells.request_revision == self.battle.data.world.revision
                        && self.battle.data.world.revision > before.data.world.revision,
            )?;
        }
        require(
            old.spells
                .histories
                .0
                .iter()
                .zip(&new.spells.histories.0)
                .all(|(a, b)| {
                    a.lane == b.lane
                        && a.ledger.casts == b.ledger.casts
                        && a.ledger.spent == b.ledger.spent
                        && a.last == b.last
                        && a.receipt == b.receipt
                }),
        )?;
        let cost = if self.reason == Reason::None {
            let r = b.receipt.as_ref().ok_or_else(invalid)?;
            let cost = i64::from(r.impact.usage.cost);
            require(
                a.ledger.casts.checked_add(1) == Some(b.ledger.casts)
                    && a.ledger.cost == r.impact.usage.cost
                    && a.ledger.spent.checked_add(cost) == Some(b.ledger.spent),
            )?;
            let x = &entry(before, expected.target.entity)?.incoming;
            let y = &entry(&self.battle, expected.target.entity)?.incoming;
            require(
                y.received > x.received
                    && i128::from(y.restored) - i128::from(x.restored)
                        >= i128::from(r.impact.target.applied),
            )?;
            cost
        } else {
            require(
                a.ledger.casts == b.ledger.casts
                    && a.ledger.spent == b.ledger.spent
                    && a.receipt == b.receipt,
            )?;
            0
        };
        require(
            old.resources
                .as_ref()
                .ok_or_else(invalid)?
                .flows
                .spent
                .checked_add(cost)
                == Some(new.resources.as_ref().ok_or_else(invalid)?.flows.spent),
        )
    }
}
/// An explicit candidate transaction, not live command authorization. Rejected
/// replies preserve both pending intent and the follower's complete current view.
pub struct Pending {
    intent: Intent,
    before: healing::Checked,
    resolved: bool,
}
impl std::fmt::Debug for Pending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingPlayerHealing[redacted]")
    }
}
impl Pending {
    pub fn begin(
        follower: &healing::Follower,
        target: i64,
        request_id: u32,
    ) -> Result<Self, app::ApplicationError> {
        let before = follower.current().ok_or_else(invalid)?.clone();
        let intent = Intent::from_candidate(&before, target, request_id)?;
        Ok(Self {
            intent,
            before,
            resolved: false,
        })
    }
    pub fn intent(&self) -> &Intent {
        &self.intent
    }
    pub fn resolved(&self) -> bool {
        self.resolved
    }
    pub fn accept(
        &mut self,
        follower: &mut healing::Follower,
        reply: &CandidateReply,
    ) -> Result<(), app::ApplicationError> {
        require(!self.resolved)?;
        reply.check_after(&self.intent, self.before.candidate())?;
        self.intent
            .check_before(follower.current().ok_or_else(invalid)?.candidate())?;
        let mut next = follower.clone();
        next.accept(reply.battle.clone())?;
        *follower = next;
        self.resolved = true;
        Ok(())
    }
}
/// One complete live transaction. A malformed, duplicate or stale response
/// preserves both the unresolved pending and the tracker's immutable snapshot.
pub(crate) struct LivePending {
    intent: Intent,
    before: super::battle_follow::Observation,
    resolved: bool,
}
impl LivePending {
    pub(crate) fn begin(
        before: &super::battle_follow::Observation,
        intent: Intent,
        peer: super::protocol::extensions::Selected,
    ) -> Result<Self, app::ApplicationError> {
        require(negotiated(peer) && intent.current(before))?;
        intent.check_before_data(before.data())?;
        Ok(Self {
            intent,
            before: before.clone(),
            resolved: false,
        })
    }
    pub(crate) fn intent(&self) -> &Intent {
        &self.intent
    }
    pub(crate) fn resolved(&self) -> bool {
        self.resolved
    }
    pub(crate) fn accept(
        &mut self,
        tracker: &mut super::battle_state::Tracker,
        reply: &Reply,
        peer: super::protocol::extensions::Selected,
    ) -> Result<(), app::ApplicationError> {
        require(!self.resolved && negotiated(peer))?;
        reply.check_after(&self.intent, self.before.data())?;
        tracker.accept_player_healing(reply, &self.intent, peer)?;
        self.resolved = true;
        Ok(())
    }
}
fn actor(data: &d::UnvalidatedData, entity: i64) -> Result<&d::Actor, app::ApplicationError> {
    data.actors
        .0
        .iter()
        .find(|a| a.entity == entity)
        .ok_or_else(invalid)
}
fn entry<'a>(
    c: &'a healing::View<'_>,
    entity: i64,
) -> Result<&'a healing::Actor, app::ApplicationError> {
    c.actors
        .0
        .iter()
        .find(|a| a.entity == entity)
        .ok_or_else(invalid)
}
fn outgoing<'a>(
    c: &'a healing::View<'_>,
    entity: i64,
) -> Result<&'a healing::Outgoing, app::ApplicationError> {
    entry(c, entity)?.outgoing.as_ref().ok_or_else(invalid)
}
fn require(ok: bool) -> Result<(), app::ApplicationError> {
    if ok {
        Ok(())
    } else {
        Err(invalid())
    }
}
fn invalid() -> app::ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid player healing contract",
    )
}
#[cfg(test)]
#[path = "battle_player_healing_tests.rs"]
mod tests;
