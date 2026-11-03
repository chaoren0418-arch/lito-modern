//! Full-VIP inspection and equipment receipts. Saved/runtime/lifecycle versions
//! remain independent; DATA-only negotiation cannot send these intents. Parsing
//! never installs state. Explicit acceptance stages a complete follower first.
use super::{
    application_protocol as app,
    battle_follow::{Follower, Observation},
    battle_operation::Lifecycle,
    battle_payload::{self as payload, data as d},
    battle_read,
    battle_window::Stamp,
    equipment,
    protocol::extensions::Selected,
    world_protocol as world,
};
use std::io::Read;

pub const REQUEST_KIND: u16 = 82;
pub const RESULT_KIND: u16 = 83;
pub const REQUEST_BYTES: usize = 156;
pub const RESULT_PREFIX_BYTES: usize = 236;
pub const MAX_RESULT_BYTES: usize = RESULT_PREFIX_BYTES + payload::vip::MAX_APPEARANCE_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Inspect,
    Set,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Rejection {
    None,
    NotActive,
    StaleLife,
    Dead,
    Unavailable,
    Busy,
    StaleSavedEquipment,
    StaleRuntimeAbility,
    StaleLifecycle,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Outcome {
    Rejected,
    Inspected,
    Unchanged,
    Applied,
    PersistedRetired,
    Conflict,
    Denied,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Request {
    pub stamp: Stamp,
    pub versions: [i64; 3],
    pub action: Action,
    pub item: i64,
    pub desired: bool,
    pub token: [u8; 32],
}
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VipEquipmentIntent[redacted]")
    }
}
impl Request {
    pub(crate) fn validate(&self) -> Result<(), app::ApplicationError> {
        let s = self.stamp;
        let v = self.versions;
        require(
            app::valid_context(s.context)
                && s.channel <= 3
                && s.world.iter().all(|n| *n > 0)
                && (1..=100).contains(&s.entity)
                && s.life > 0
                && s.revision > 0
                && v[0] >= 0
                && v[1] >= v[0]
                && v[2] >= 0
                && match self.action {
                    Action::Inspect => self.item == 0 && !self.desired && self.token == [0; 32],
                    Action::Set => self.item > 0,
                },
        )
    }
    pub fn inspect(o: &Observation) -> Result<Self, app::ApplicationError> {
        require(active_alive(o))?;
        let q = Self {
            stamp: Stamp::of(o),
            versions: versions(o.data())?,
            action: Action::Inspect,
            item: 0,
            desired: false,
            token: [0; 32],
        };
        q.validate()?;
        Ok(q)
    }
    pub(crate) fn current(&self, o: &Observation) -> bool {
        active_alive(o)
            && self.stamp.current(o)
            && versions(o.data()).is_ok_and(|v| v == self.versions)
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
        d::WorldId {
            room: self.stamp.world[0],
            match_id: self.stamp.world[1],
            generation: self.stamp.world[2],
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Quote {
    stamp: Stamp,
    versions: [i64; 3],
    token: [u8; 32],
}
impl std::fmt::Debug for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VipEquipmentQuote[redacted]")
    }
}
impl Quote {
    pub(crate) fn current(&self, o: &Observation) -> bool {
        active_alive(o)
            && self.stamp.current(o)
            && versions(o.data()).is_ok_and(|v| v == self.versions)
    }
    /// Source token is server issued, never synthesized from displayed stats.
    /// A newer same-state observation may advance world revision, not identity,
    /// life or any of the three versions. The server still checks expiry at
    /// operation admission and durable commit; this quote is not a time grant.
    pub fn set(
        &self,
        o: &Observation,
        item: i64,
        desired: bool,
    ) -> Result<Request, app::ApplicationError> {
        require(active_alive(o) && self.stamp.current(o) && self.versions == versions(o.data())?)?;
        let q = Request {
            stamp: Stamp::of(o),
            versions: self.versions,
            action: Action::Set,
            item,
            desired,
            token: self.token,
        };
        q.validate()?;
        Ok(q)
    }
}
fn active_alive(o: &Observation) -> bool {
    o.data().vip.is_some()
        && o.data().world.phase == d::Phase::Active
        && o.data().actors.0.iter().any(|a| {
            a.entity == o.data().personal.entity
                && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
        })
}
fn versions(data: &payload::UnvalidatedData) -> Result<[i64; 3], app::ApplicationError> {
    let v = &data.vip.as_ref().ok_or_else(invalid)?.owner.versions;
    Ok([v.saved, v.runtime, v.lifecycle])
}

#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub rejection: Rejection,
    pub outcome: Outcome,
    pub denial: Option<equipment::Reason>,
    pub blocking_item: i64,
    pub durable_revision: i64,
    pub token: Option<[u8; 32]>,
    pub lifecycle: Lifecycle,
    pub data: payload::UnvalidatedData,
}
impl std::fmt::Debug for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VipEquipmentReceipt[redacted]")
    }
}
impl Reply {
    pub(crate) fn validate(&self) -> Result<(), app::ApplicationError> {
        let q = self.echo;
        q.validate()?;
        self.data.check_scene_values(&q.player(), &q.world())?;
        self.lifecycle.validate(&self.data)?;
        let v = versions(&self.data)?;
        let seen = q.versions;
        let s = q.stamp;
        let owner = &self.data.vip.as_ref().ok_or_else(invalid)?.owner;
        let life = owner.life.checked_add(1).ok_or_else(invalid)?;
        let actor = self
            .data
            .actors
            .0
            .iter()
            .find(|a| a.entity == s.entity)
            .ok_or_else(invalid)?;
        let hp = actor.vital.as_ref().ok_or_else(invalid)?.values.hp;
        require(
            self.data.personal.entity == s.entity
                && self.data.world.revision >= s.revision
                && self.data.race.is_none()
                && self.blocking_item >= 0
                && (self.outcome == Outcome::Denied) == self.denial.is_some()
                && (self.outcome == Outcome::Denied || self.blocking_item == 0)
                && (self.outcome == Outcome::Rejected) == (self.rejection != Rejection::None)
                && matches!(
                    self.outcome,
                    Outcome::Inspected | Outcome::Unchanged | Outcome::Applied
                ) == self.token.is_some(),
        )?;
        if self.outcome == Outcome::Rejected {
            require(self.durable_revision == -1)?;
            return require(match self.rejection {
                Rejection::NotActive => {
                    self.data.world.phase != d::Phase::Active
                        || self.lifecycle.phase != world::Phase::Active
                }
                Rejection::StaleLife => life != s.life,
                Rejection::Dead => life == s.life && hp == 0,
                Rejection::StaleSavedEquipment => seen[0] != v[0],
                Rejection::StaleRuntimeAbility => seen[0] == v[0] && seen[1] != v[1],
                Rejection::StaleLifecycle => seen[0] == v[0] && seen[1] == v[1] && seen[2] != v[2],
                Rejection::Unavailable | Rejection::Busy => true,
                Rejection::None => false,
            });
        }
        require((self.outcome == Outcome::Inspected) == (q.action == Action::Inspect))?;
        match self.outcome {
            Outcome::Inspected | Outcome::Unchanged | Outcome::Applied => {
                let applied = self.outcome == Outcome::Applied;
                let expected = if applied {
                    [
                        seen[0].checked_add(1).ok_or_else(invalid)?,
                        seen[1].checked_add(1).ok_or_else(invalid)?,
                        seen[2].checked_add(1).ok_or_else(invalid)?,
                    ]
                } else {
                    seen
                };
                require(
                    v == expected
                        && life == s.life
                        && self.data.world.phase == d::Phase::Active
                        && self.lifecycle.phase == world::Phase::Active
                        && (applied || hp > 0)
                        && self.durable_revision
                            == if self.outcome == Outcome::Inspected {
                                -1
                            } else {
                                v[0]
                            }
                        && (!applied || self.data.world.revision > s.revision),
                )?;
                if q.action == Action::Set {
                    let e = self.data.personal.equipment.as_ref().ok_or_else(invalid)?;
                    require(
                        e.equipped
                            .0
                            .iter()
                            .any(|e| e.object == q.item && e.equipped == q.desired),
                    )?;
                }
                Ok(())
            }
            Outcome::PersistedRetired => require(
                seen[0].checked_add(1) == Some(self.durable_revision)
                    && v[0] == seen[0]
                    && matches!(
                        self.lifecycle.phase,
                        world::Phase::Aborted | world::Phase::Finished
                    ),
            ),
            Outcome::Conflict | Outcome::Denied => require(self.durable_revision == -1),
            Outcome::Rejected => Err(invalid()),
        }
    }
    /// No optimistic UI/store acknowledgement. Validate intent, receipt and
    /// complete successor before changing the supplied observation follower.
    pub fn accept(
        &self,
        expected: &Request,
        follower: &mut Follower,
    ) -> Result<Option<Quote>, app::ApplicationError> {
        require(&self.echo == expected)?;
        self.validate()?;
        let old = follower.observation().ok_or_else(invalid)?;
        let stamp = Stamp::of(old);
        let q = expected.stamp;
        require(
            stamp.context == q.context
                && stamp.channel == q.channel
                && stamp.world == q.world
                && stamp.entity == q.entity
                && stamp.revision >= q.revision,
        )?;
        let mut next = follower.clone();
        next.accept(battle_read::UnvalidatedReply {
            echo: battle_read::Request {
                context: q.context,
                channel_id: q.channel,
                world: expected.world(),
            },
            state: battle_read::State::Data,
            data: Some(self.data.clone()),
        })?;
        let o = next.observation().ok_or_else(invalid)?;
        let current_versions = versions(o.data())?;
        let quote = self.token.map(|token| Quote {
            stamp: Stamp::of(o),
            versions: current_versions,
            token,
        });
        *follower = next;
        Ok(quote)
    }
}
pub fn encode_request(
    id: u32,
    q: &Request,
    peer: Selected,
) -> Result<Vec<u8>, app::ApplicationError> {
    require(peer.allows_vip_equipment())?;
    q.validate()?;
    let s = q.stamp;
    let mut b = app::header(id, REQUEST_KIND, REQUEST_BYTES)?;
    b.extend(s.context.key.source_id.to_be_bytes());
    b.extend(s.context.key.object_id.to_be_bytes());
    b.extend(s.context.epoch.to_be_bytes());
    b.extend(u32::from(s.channel).to_be_bytes());
    for n in [
        s.world[0],
        s.world[1],
        s.world[2],
        s.entity,
        s.life,
        s.revision,
        q.versions[0],
        q.versions[1],
        q.versions[2],
    ] {
        b.extend(n.to_be_bytes());
    }
    b.extend([q.action as u8, u8::from(q.desired), 0, 0]);
    b.extend(q.item.to_be_bytes());
    b.extend(q.token);
    Ok(b)
}
pub(crate) fn decode_request(b: &[u8], peer: Selected) -> Result<Request, app::ApplicationError> {
    require(peer.allows_vip_equipment() && b.len() == REQUEST_BYTES)?;
    require(b[112] <= 1 && b[113] <= 1 && b[114..116] == [0; 2])?;
    let q = Request {
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: u32_at(b, 16),
                    object_id: i64_at(b, 20) as u64,
                },
                epoch: i64_at(b, 28) as u64,
            },
            channel: u32_at(b, 36).try_into().map_err(|_| invalid())?,
            world: [i64_at(b, 40), i64_at(b, 48), i64_at(b, 56)],
            entity: i64_at(b, 64),
            life: i64_at(b, 72),
            revision: i64_at(b, 80),
        },
        versions: [i64_at(b, 88), i64_at(b, 96), i64_at(b, 104)],
        action: if b[112] == 0 {
            Action::Inspect
        } else {
            Action::Set
        },
        desired: b[113] == 1,
        item: i64_at(b, 116),
        token: b[124..156].try_into().unwrap(),
    };
    require(encode_request(u32_at(b, 8), &q, peer)? == b)?;
    Ok(q)
}
pub fn read_reply(
    reader: &mut impl Read,
    expected: &[u8],
    peer: Selected,
) -> Result<Reply, app::ApplicationError> {
    let q = decode_request(expected, peer)?;
    let mut h = [0; 16];
    reader.read_exact(&mut h).map_err(|_| invalid())?;
    let size = u32_at(&h, 4) as usize;
    require(
        &h[..4] == b"LTM1"
            && h[8..12] == expected[8..12]
            && h[12..14] == RESULT_KIND.to_be_bytes()
            && h[14..16] == [0; 2]
            && (RESULT_PREFIX_BYTES..=MAX_RESULT_BYTES).contains(&size),
    )?;
    let mut body = vec![0; size - 16];
    reader.read_exact(&mut body).map_err(|_| invalid())?;
    require(body[..REQUEST_BYTES - 16] == expected[16..])?;
    let m = &body[REQUEST_BYTES - 16..RESULT_PREFIX_BYTES - 16];
    require(m[3] == 0 && u32_at(m, 76) as usize == size - RESULT_PREFIX_BYTES)?;
    let rejection = *[
        Rejection::None,
        Rejection::NotActive,
        Rejection::StaleLife,
        Rejection::Dead,
        Rejection::Unavailable,
        Rejection::Busy,
        Rejection::StaleSavedEquipment,
        Rejection::StaleRuntimeAbility,
        Rejection::StaleLifecycle,
    ]
    .get(m[0] as usize)
    .ok_or_else(invalid)?;
    let outcome = *[
        Outcome::Rejected,
        Outcome::Inspected,
        Outcome::Unchanged,
        Outcome::Applied,
        Outcome::PersistedRetired,
        Outcome::Conflict,
        Outcome::Denied,
    ]
    .get(m[1] as usize)
    .ok_or_else(invalid)?;
    let has_token = matches!(
        outcome,
        Outcome::Inspected | Outcome::Unchanged | Outcome::Applied
    );
    require(has_token || m[20..52] == [0; 32])?;
    let r = Reply {
        echo: q,
        rejection,
        outcome,
        denial: denial(m[2])?,
        blocking_item: i64_at(m, 4),
        durable_revision: i64_at(m, 12),
        token: has_token.then(|| m[20..52].try_into().unwrap()),
        lifecycle: Lifecycle::read(&m[52..76])?,
        data: payload::decode_selected(&body[RESULT_PREFIX_BYTES - 16..], peer)?,
    };
    r.validate()?;
    Ok(r)
}
pub(crate) fn encode_reply(
    id: u32,
    r: &Reply,
    peer: Selected,
) -> Result<Vec<u8>, app::ApplicationError> {
    require(peer.allows_vip_equipment() && battle_read::data_negotiated_selected(&r.data, peer))?;
    r.validate()?;
    let data = payload::encode(&r.data)?;
    let size = RESULT_PREFIX_BYTES
        .checked_add(data.len())
        .ok_or_else(invalid)?;
    require(size <= MAX_RESULT_BYTES)?;
    let mut b = encode_request(id, &r.echo, peer)?;
    b[4..8].copy_from_slice(&(size as u32).to_be_bytes());
    b[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
    b.extend([
        r.rejection as u8,
        r.outcome as u8,
        r.denial.map_or(0, |d| d as u8),
        0,
    ]);
    b.extend(r.blocking_item.to_be_bytes());
    b.extend(r.durable_revision.to_be_bytes());
    b.extend(r.token.unwrap_or([0; 32]));
    let l = r.lifecycle;
    b.extend(l.revision.to_be_bytes());
    b.extend([
        l.phase as u8,
        l.abort as u8,
        l.settlement.map_or(0, |s| s as u8 + 1),
        l.race_settlement.map_or(0, |s| s as u8 + 1),
    ]);
    b.extend(l.race_terminal_revision.to_be_bytes());
    b.extend([0; 4]);
    b.extend((data.len() as u32).to_be_bytes());
    b.extend(data);
    Ok(b)
}
fn denial(code: u8) -> Result<Option<equipment::Reason>, app::ApplicationError> {
    use equipment::Reason::*;
    if code == 0 {
        return Ok(None);
    };
    [
        InvalidRequest,
        UnknownItem,
        UnqualifiedItem,
        NotEquipment,
        UnqualifiedEquippedItem,
        InvalidLoadout,
        ClassNotAllowed,
        LevelTooLow,
        LevelTooHigh,
        Cursed,
        Sealed,
        SlotOccupied,
        RingLimit,
        AidLimit,
        DuplicateAidCatalog,
        TalismanLimit,
        ShieldGuarderConflict,
        TwoHandedShieldConflict,
        LayerDependency,
    ]
    .get(code as usize - 1)
    .copied()
    .map(Some)
    .ok_or_else(invalid)
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}
fn i64_at(b: &[u8], at: usize) -> i64 {
    i64::from_be_bytes(b[at..at + 8].try_into().unwrap())
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
        "invalid full VIP equipment receipt",
    )
}

#[cfg(test)]
#[path = "battle_vip_equipment_tests.rs"]
mod tests;
