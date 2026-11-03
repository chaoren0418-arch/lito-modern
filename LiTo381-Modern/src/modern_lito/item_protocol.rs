//! v8 atomic use of server-owned match-local healing stacks. No source object
//! IDs, local HP edits, automatic replay, or source inventory writes.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    combat_protocol as combat,
    world_map::WorldCell,
    world_protocol as world,
};
pub const REQUEST_KIND: u16 = 34;
pub const RESULT_KIND: u16 = 35;
pub const MAX_FRAME_BYTES: usize = combat::MAX_FRAME_BYTES;
pub const MAX_STOCKS: usize = 7;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Use,
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
    NotActive,
    Dead,
    Empty,
    TooFast,
    Unsupported,
    Numeric,
    ClassRestricted,
    StaleRegen,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealingKind {
    pub item_id: i64,
    pub minimum: i64,
    pub maximum: i64,
    pub effect: i32,
    pub delay_millis: i32,
    pub inventory_graphic: i64,
}
pub const HEALING: [HealingKind; 3] = [
    HealingKind {
        item_id: 40010,
        minimum: 10,
        maximum: 20,
        effect: 189,
        delay_millis: 500,
        inventory_graphic: 80,
    },
    HealingKind {
        item_id: 40011,
        minimum: 30,
        maximum: 50,
        effect: 194,
        delay_millis: 1000,
        inventory_graphic: 78,
    },
    HealingKind {
        item_id: 40012,
        minimum: 60,
        maximum: 80,
        effect: 197,
        delay_millis: 1000,
        inventory_graphic: 81,
    },
];
pub fn healing(id: i64) -> Option<&'static HealingKind> {
    HEALING.iter().find(|k| k.item_id == id)
}
pub fn supported(id: i64) -> bool {
    healing(id).is_some() || super::timed_protocol::Kind::from_item(id).is_some()
}
pub const UI_ITEMS: [i64; 7] = [40010, 40011, 40012, 40013, 40014, 40068, 40016];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub action: Action,
    pub match_id: u64,
    pub generation: u64,
    pub sequence: u64,
    pub item_id: i64,
}
impl Request {
    pub fn read(context: SelectionContext, channel_id: u8, frame: &world::Frame) -> Self {
        Self {
            context,
            channel_id,
            action: Action::Read,
            match_id: frame.match_id,
            generation: frame.generation,
            sequence: 0,
            item_id: 0,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || !positive(self.match_id)
            || !positive(self.generation)
            || match self.action {
                Action::Read => self.sequence != 0 || self.item_id != 0,
                Action::Use => !positive(self.sequence) || self.item_id < 1,
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stock {
    pub item_id: i64,
    pub initial: i64,
    pub remaining: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemState {
    pub entity_id: u64,
    pub request_sequence: u64,
    pub use_count: u64,
    pub use_revision: i64,
    pub item_id: i64,
    pub restored: i64,
    pub hp_before: i64,
    pub hp_after: i64,
    pub effect: i32,
    pub delay_millis: i32,
    pub position: WorldCell,
    pub fixed_recovery: i64,
    pub stocks: Vec<Stock>,
}
impl ItemState {
    pub fn stock(&self, id: i64) -> Option<&Stock> {
        self.stocks.iter().find(|s| s.item_id == id)
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !positive(self.entity_id)
            || self.request_sequence > i64::MAX as u64
            || self.use_count > self.request_sequence
            || self.use_revision < 0
            || self.restored < 0
            || self.hp_before < 0
            || self.hp_after < 0
            || self.fixed_recovery < 0
            || self.stocks.len() > MAX_STOCKS
        {
            return Err(invalid());
        }
        let mut last = 0;
        let mut consumed = 0u64;
        for s in &self.stocks {
            if s.item_id <= last
                || !supported(s.item_id)
                || !(1..=1_000_000).contains(&s.initial)
                || !(0..=s.initial).contains(&s.remaining)
            {
                return Err(invalid());
            }
            consumed = consumed
                .checked_add((s.initial - s.remaining) as u64)
                .ok_or_else(invalid)?;
            last = s.item_id;
        }
        if consumed != self.use_count {
            return Err(invalid());
        }
        if self.use_count == 0 {
            if self.use_revision != 0
                || self.item_id != 0
                || self.restored != 0
                || self.hp_before != 0
                || self.hp_after != 0
                || self.effect != 0
                || self.delay_millis != 0
                || self.position != (WorldCell { x: 0, y: 0 })
            {
                return Err(invalid());
            }
        } else {
            let healing = healing(self.item_id);
            let timed = super::timed_protocol::Kind::from_item(self.item_id);
            let (effect, delay) = if let Some(k) = healing {
                (k.effect, k.delay_millis)
            } else if let Some(k) = timed {
                (k.effect(), 0)
            } else {
                return Err(invalid());
            };
            if self.use_revision == 0
                || self.hp_before == 0
                || self.hp_after.checked_sub(self.hp_before) != Some(self.restored)
                || self.effect != effect
                || self.delay_millis != delay
                || healing.is_some_and(|k| {
                    self.restored > k.maximum && self.restored - k.maximum > self.fixed_recovery
                })
                || timed.is_some() && self.restored != 0
                || !self
                    .stock(self.item_id)
                    .is_some_and(|s| s.remaining < s.initial)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
pub fn validate_states(frame: &combat::Frame) -> Result<(), ApplicationError> {
    if frame.items.is_empty() {
        return Ok(());
    }
    if frame.items.len() != frame.world.actors.len() || frame.items.len() != frame.vitals.len() {
        return Err(invalid());
    }
    for ((state, actor), vital) in frame
        .items
        .iter()
        .zip(&frame.world.actors)
        .zip(&frame.vitals)
    {
        state.validate()?;
        if let Some(k) =
            healing(state.item_id).filter(|_| state.use_count > 0 && state.hp_after < vital.max_hp)
        {
            let min = k.minimum;
            if state.restored < min || state.restored - min < state.fixed_recovery {
                return Err(invalid());
            }
        }
        if state.entity_id != actor.entity_id
            || state.use_revision > frame.world.revision
            || state.hp_after > vital.max_hp
            || actor.identity().character().is_none()
                && (!state.stocks.is_empty()
                    || state.request_sequence != 0
                    || state.fixed_recovery != 0)
            || state.use_count > 0
                && (frame.regen.is_empty()
                    && (vital.hp > state.hp_after
                        || vital.hurt_revision <= state.use_revision && vital.hp != state.hp_after)
                    || !frame.world.manifest.contains(state.position)
                    || state.use_revision == frame.world.revision
                        && (vital.hp != state.hp_after || actor.position != state.position))
            || frame.world.completion.as_ref().is_some_and(|c| {
                c.event
                    .engine_revision
                    .checked_add(1)
                    .is_none_or(|rev| state.use_revision > rev)
            })
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub fn states_follow(a: &combat::Frame, b: &combat::Frame) -> bool {
    if a.items.len() != b.items.len() {
        return false;
    }
    a.items.iter().zip(&b.items).all(|(old, new)| {
        if old.entity_id != new.entity_id
            || old.fixed_recovery != new.fixed_recovery
            || new.request_sequence < old.request_sequence
            || new.use_count < old.use_count
            || new.use_revision < old.use_revision
            || new.use_count - old.use_count > new.request_sequence - old.request_sequence
            || old.stocks.len() != new.stocks.len()
        {
            return false;
        }
        if !old.stocks.iter().zip(&new.stocks).all(|(a, b)| {
            a.item_id == b.item_id && a.initial == b.initial && b.remaining <= a.remaining
        }) {
            return false;
        }
        if new.use_count == old.use_count {
            let mut without_seq = new.clone();
            without_seq.request_sequence = old.request_sequence;
            without_seq == *old
        } else {
            new.use_revision > old.use_revision
                && new.use_revision > a.world.revision
                && old
                    .stock(new.item_id)
                    .zip(new.stock(new.item_id))
                    .is_some_and(|(old, new)| new.remaining < old.remaining)
        }
    })
}
/// Healing is the only admitted HP increase. Coalesced reads can contain several
/// uses plus incoming damage; stock deltas bound the net increase without inventing
/// unreported intermediate RNG rolls. A dead actor still cannot be revived.
pub fn hp_follows(a: &combat::Frame, b: &combat::Frame, index: usize) -> bool {
    if !a.regen.is_empty() || !b.regen.is_empty() {
        return super::regen_protocol::hp_follows(a, b, index);
    }
    let old = &a.vitals[index];
    let new = &b.vitals[index];
    if super::siege_protocol::respawned(&a.world, &b.world, old.entity_id) {
        // Siege may be configured without REGEN. Its separate, validated life
        // ledger grants full HP on each server respawn, not a consumable heal.
        return new.hp == new.max_hp || new.hurt_revision > old.hurt_revision;
    }
    if let (Some(before), Some(after)) = (a.items.get(index), b.items.get(index)) {
        if after.use_count > before.use_count
            && new.hurt_revision == old.hurt_revision
            && after.hp_before < old.hp
        {
            return false;
        }
    }
    if new.hp < old.hp && new.hurt_revision <= old.hurt_revision {
        return false;
    }
    if new.hp <= old.hp {
        return true;
    }
    let (Some(before), Some(after)) = (a.items.get(index), b.items.get(index)) else {
        return false;
    };
    if old.hp == 0
        || after.use_count <= before.use_count
        || after.use_revision <= before.use_revision
    {
        return false;
    }
    let maximum = before
        .stocks
        .iter()
        .zip(&after.stocks)
        .try_fold(0i128, |sum, (a, b)| {
            let used = a.remaining.checked_sub(b.remaining)?;
            let bound = healing(a.item_id).map_or(0, |k| {
                i128::from(k.maximum) + i128::from(after.fixed_recovery)
            });
            sum.checked_add(i128::from(used).checked_mul(bound)?)
        });
    // Earlier skipped rolls have only bounded maxima. The newest consumed
    // bottle has an exact reported restored amount, so never count its unused
    // theoretical capacity as permission for an extra, unreported HP increase.
    let maximum = maximum.and_then(|sum| {
        if let Some(k) = healing(after.item_id) {
            sum.checked_sub(i128::from(k.maximum) + i128::from(after.fixed_recovery))?
                .checked_add(i128::from(after.restored))
        } else {
            Some(sum)
        }
    });
    maximum.is_some_and(|n| {
        new.hp
            .checked_sub(old.hp)
            .is_some_and(|delta| i128::from(delta) <= n)
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: Status,
    pub reason: Reason,
    pub frame: Option<combat::Frame>,
}
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        if (self.status == Status::Ok) != (self.reason == Reason::None) {
            return Err(invalid());
        }
        let Some(f) = &self.frame else {
            return if self.status == Status::Unavailable && self.reason == Reason::NotActive {
                Ok(())
            } else {
                Err(invalid())
            };
        };
        f.validate()?;
        if f.world.match_id != self.echo.match_id
            || f.world.generation != self.echo.generation
            || f.items.is_empty()
        {
            return Err(invalid());
        }
        let a = f
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(self.echo.context.key))
            .ok_or_else(invalid)?;
        let state = f.item(a.entity_id).ok_or_else(invalid)?;
        if self.reason == Reason::StaleRegen
            && (self.status != Status::Denied
                || self.echo.action != Action::Use
                || f.world.phase != world::Phase::Active
                || healing(self.echo.item_id).is_none()
                || state.request_sequence != self.echo.sequence
                || f.vital(a.entity_id).is_none_or(|v| v.hp <= 0)
                || state
                    .stock(self.echo.item_id)
                    .is_none_or(|s| s.remaining <= 0)
                || f.regen(a.entity_id).is_none_or(|r| r.hp_revision <= 0))
        {
            return Err(invalid());
        }
        if self.reason == Reason::ClassRestricted {
            let kind =
                super::timed_protocol::Kind::from_item(self.echo.item_id).ok_or_else(invalid)?;
            let member = f
                .world
                .room
                .members
                .iter()
                .find(|m| m.matches_character(self.echo.context.key))
                .ok_or_else(invalid)?;
            if self.status != Status::Denied
                || self.echo.action != Action::Use
                || f.world.phase != world::Phase::Active
                || f.timed.is_empty()
                || state.request_sequence != self.echo.sequence
                || kind.eligible(member.class)
                || f.vital(a.entity_id).is_none_or(|v| v.hp <= 0)
                || state
                    .stock(self.echo.item_id)
                    .is_none_or(|s| s.remaining <= 0)
            {
                return Err(invalid());
            }
        }
        if self.echo.action == Action::Use
            && self.status == Status::Ok
            && (f.world.phase != world::Phase::Active
                || state.request_sequence != self.echo.sequence
                || state.use_revision != f.world.revision
                || state.item_id != self.echo.item_id
                || f.vital(a.entity_id).ok_or_else(invalid)?.hp != state.hp_after)
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn combat_reply(&self) -> combat::Reply {
        combat::Reply {
            echo: combat::Request {
                context: self.echo.context,
                channel_id: self.echo.channel_id,
                action: combat::Action::Read,
                match_id: self.echo.match_id,
                generation: self.echo.generation,
                sequence: 0,
                target_entity_id: 0,
            },
            status: if self.frame.is_some() {
                combat::Status::Ok
            } else {
                combat::Status::Unavailable
            },
            reason: if self.frame.is_some() {
                combat::Reason::None
            } else {
                combat::Reason::NotActive
            },
            frame: self.frame.clone(),
        }
    }
    pub fn confirms(&self, request: &Request, prior: Option<&combat::Reply>) -> bool {
        if self.echo != *request
            || self.validate().is_err()
            || !combat::follows(prior, &self.combat_reply())
        {
            return false;
        }
        if request.action == Action::Read {
            return true;
        }
        let Some(f) = self.frame.as_ref() else {
            return true;
        };
        let Some(actor) = f
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(request.context.key))
        else {
            return false;
        };
        let Some(after) = f.item(actor.entity_id) else {
            return false;
        };
        let Some(before) = prior
            .and_then(|r| r.frame.as_ref())
            .and_then(|f| f.item(actor.entity_id))
        else {
            return false;
        };
        if self.status == Status::Ok {
            if before.use_count.checked_add(1) != Some(after.use_count)
                || after.request_sequence != request.sequence
            {
                return false;
            }
            before.stocks.iter().zip(&after.stocks).all(|(a, b)| {
                a.item_id == b.item_id
                    && a.initial == b.initial
                    && a.remaining
                        .checked_sub(i64::from(a.item_id == request.item_id))
                        == Some(b.remaining)
            })
        } else {
            after.use_count == before.use_count
                && after.use_revision == before.use_revision
                && after.stocks == before.stocks
                && (after.request_sequence == request.sequence
                    || matches!(
                        f.world.phase,
                        world::Phase::Aborted | world::Phase::Finished
                    ) && after.request_sequence == before.request_sequence)
        }
    }
}
fn positive(n: u64) -> bool {
    n > 0 && n <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid match-local item frame",
    )
}
pub fn encode_request(id: u32, r: Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    if !(8..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    r.validate()?;
    let mut bytes = app::header(id, REQUEST_KIND, 72)?;
    bytes.extend(r.context.key.source_id.to_be_bytes());
    bytes.extend(r.context.key.object_id.to_be_bytes());
    bytes.extend(r.context.epoch.to_be_bytes());
    bytes.extend([r.channel_id, r.action as u8, 0, 0]);
    bytes.extend(r.match_id.to_be_bytes());
    bytes.extend(r.generation.to_be_bytes());
    bytes.extend(r.sequence.to_be_bytes());
    bytes.extend(r.item_id.to_be_bytes());
    Ok(bytes)
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.at.checked_add(n).ok_or_else(invalid)?;
        let value = self.bytes.get(self.at..end).ok_or_else(invalid)?;
        self.at = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, ApplicationError> {
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
}
#[cfg(test)]
pub(crate) fn decode_states(
    bytes: &[u8],
    actor_count: usize,
) -> Result<(Vec<ItemState>, usize), ApplicationError> {
    decode_states_version(bytes, actor_count, 8)
}
pub(crate) fn decode_states_version(
    bytes: &[u8],
    actor_count: usize,
    version: u16,
) -> Result<(Vec<ItemState>, usize), ApplicationError> {
    let mut r = Reader { bytes, at: 0 };
    let count = r.u16()? as usize;
    r.zero(2)?;
    if count > 100 || count != 0 && count != actor_count || bytes.len() - r.at < count * 92 {
        return Err(invalid());
    }
    let mut states = Vec::with_capacity(count);
    for _ in 0..count {
        let entity_id = r.u64()?;
        let request_sequence = r.u64()?;
        let use_count = r.u64()?;
        let use_revision = r.i64()?;
        let item_id = r.i64()?;
        let restored = r.i64()?;
        let hp_before = r.i64()?;
        let hp_after = r.i64()?;
        let effect = r.u32()? as i32;
        let delay_millis = r.u32()? as i32;
        let position = WorldCell {
            x: r.u32()? as i32,
            y: r.u32()? as i32,
        };
        let fixed_recovery = r.i64()?;
        let count = r.u16()? as usize;
        r.zero(2)?;
        if count > if version >= 10 { MAX_STOCKS } else { 3 } || bytes.len() - r.at < count * 24 {
            return Err(invalid());
        }
        let mut stocks = Vec::with_capacity(count);
        for _ in 0..count {
            stocks.push(Stock {
                item_id: r.i64()?,
                initial: r.i64()?,
                remaining: r.i64()?,
            });
        }
        let state = ItemState {
            entity_id,
            request_sequence,
            use_count,
            use_revision,
            item_id,
            restored,
            hp_before,
            hp_after,
            effect,
            delay_millis,
            position,
            fixed_recovery,
            stocks,
        };
        state.validate()?;
        if version < 10
            && (super::timed_protocol::Kind::from_item(state.item_id).is_some()
                || state.stocks.iter().any(|s| healing(s.item_id).is_none()))
        {
            return Err(invalid());
        }
        states.push(state);
    }
    Ok((states, r.at))
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(8..=super::protocol::MAX_VERSION).contains(&version)
        || bytes.len() < 60
        || bytes.len() > combat::frame_limit(version) - 16
    {
        return Err(invalid());
    }
    let mut r = Reader { bytes, at: 0 };
    let context = SelectionContext {
        key: app::CharacterKey {
            source_id: r.u32()?,
            object_id: r.u64()?,
        },
        epoch: r.u64()?,
    };
    let channel_id = r.byte()?;
    let action = match r.byte()? {
        0 => Action::Read,
        1 => Action::Use,
        _ => return Err(invalid()),
    };
    r.zero(2)?;
    let echo = Request {
        context,
        channel_id,
        action,
        match_id: r.u64()?,
        generation: r.u64()?,
        sequence: r.u64()?,
        item_id: r.i64()?,
    };
    let status = match r.byte()? {
        0 => Status::Ok,
        1 => Status::Denied,
        2 => Status::Busy,
        3 => Status::Unavailable,
        _ => return Err(invalid()),
    };
    let reason = match r.byte()? {
        0 => Reason::None,
        1 => Reason::NotActive,
        2 => Reason::Dead,
        3 => Reason::Empty,
        4 => Reason::TooFast,
        5 => Reason::Unsupported,
        6 => Reason::Numeric,
        7 if version >= 10 => Reason::ClassRestricted,
        8 if version >= 12 => Reason::StaleRegen,
        _ => return Err(invalid()),
    };
    let present = r.byte()?;
    r.zero(1)?;
    let frame = match present {
        0 => None,
        1 => {
            let (f, n) = combat::decode_frame_version(&bytes[r.at..], context.key, version)?;
            r.take(n)?;
            Some(f)
        }
        _ => return Err(invalid()),
    };
    if r.at != bytes.len() {
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn initial() -> combat::Reply {
        let mut r = super::super::combat_protocol::tests::active_reply();
        let f = r.frame.as_mut().unwrap();
        for v in &mut f.vitals {
            v.hp = 30;
            v.max_hp = 100;
        }
        f.items = f
            .world
            .actors
            .iter()
            .map(|a| ItemState {
                entity_id: a.entity_id,
                request_sequence: 0,
                use_count: 0,
                use_revision: 0,
                item_id: 0,
                restored: 0,
                hp_before: 0,
                hp_after: 0,
                effect: 0,
                delay_millis: 0,
                position: WorldCell { x: 0, y: 0 },
                fixed_recovery: 0,
                stocks: if a.entity_id == 1 {
                    vec![
                        Stock {
                            item_id: 40010,
                            initial: 3,
                            remaining: 3,
                        },
                        Stock {
                            item_id: 40011,
                            initial: 1,
                            remaining: 1,
                        },
                    ]
                } else {
                    Vec::new()
                },
            })
            .collect();
        r.validate().unwrap();
        r
    }
    pub(crate) fn used(before: &combat::Reply) -> Reply {
        let old = before.frame.as_ref().unwrap();
        let mut f = old.clone();
        f.world.revision += 1;
        let s = &mut f.items[0];
        s.request_sequence += 1;
        s.use_count += 1;
        s.use_revision = f.world.revision;
        s.item_id = 40010;
        s.restored = 10;
        s.hp_before = f.vitals[0].hp;
        s.hp_after = s.hp_before + 10;
        s.effect = 189;
        s.delay_millis = 500;
        s.position = f.world.actors[0].position;
        s.stocks[0].remaining -= 1;
        f.vitals[0].hp = s.hp_after;
        Reply {
            echo: Request {
                context: before.echo.context,
                channel_id: before.echo.channel_id,
                action: Action::Use,
                match_id: f.world.match_id,
                generation: f.world.generation,
                sequence: s.request_sequence,
                item_id: 40010,
            },
            status: Status::Ok,
            reason: Reason::None,
            frame: Some(f),
        }
    }
    pub(crate) fn state_bytes(states: &[ItemState]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend((states.len() as u16).to_be_bytes());
        b.extend([0; 2]);
        for s in states {
            for n in [
                s.entity_id,
                s.request_sequence,
                s.use_count,
                s.use_revision as u64,
                s.item_id as u64,
                s.restored as u64,
                s.hp_before as u64,
                s.hp_after as u64,
            ] {
                b.extend(n.to_be_bytes());
            }
            for n in [s.effect, s.delay_millis, s.position.x, s.position.y] {
                b.extend(n.to_be_bytes());
            }
            b.extend(s.fixed_recovery.to_be_bytes());
            b.extend((s.stocks.len() as u16).to_be_bytes());
            b.extend([0; 2]);
            for stock in &s.stocks {
                for n in [stock.item_id, stock.initial, stock.remaining] {
                    b.extend(n.to_be_bytes());
                }
            }
        }
        b
    }
    #[test]
    fn request_exact_layout_and_version_gate_no_source_item_id() {
        let r = Request {
            context: SelectionContext {
                key: app::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: u64::MAX,
            },
            channel_id: 2,
            action: Action::Use,
            match_id: 2,
            generation: 3,
            sequence: 4,
            item_id: 40010,
        };
        let bytes = encode_request(0xfedcba98, r, 8).unwrap();
        let expected = "4c544d3100000048fedcba9800220000000000070000000000000d05ffffffffffffffff020100000000000000000002000000000000000300000000000000040000000000009c4a";
        let expected: Vec<u8> = expected
            .as_bytes()
            .chunks_exact(2)
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect();
        assert_eq!(bytes, expected);
        for v in 1..=7 {
            assert!(encode_request(1, r, v).is_err());
        }
        let mut invalid = r;
        invalid.context.epoch = 0;
        assert!(invalid.validate().is_err());
        invalid = r;
        invalid.sequence = 0;
        assert!(invalid.validate().is_err());
    }
    #[test]
    fn full_states_exact_ninety_two_byte_prefix_and_retained_zero_stocks() {
        let before = initial();
        let after = used(&before);
        let f = after.frame.unwrap();
        let bytes = state_bytes(&f.items);
        assert_eq!(bytes.len(), 4 + 2 * 92 + 2 * 24);
        assert_eq!(
            decode_states(&bytes, 2).unwrap(),
            (f.items.clone(), bytes.len())
        );
        for n in 0..bytes.len() {
            assert!(decode_states(&bytes[..n], 2).is_err(), "prefix {n}");
        }
        let mut bad = bytes.clone();
        bad[2] = 1;
        assert!(decode_states(&bad, 2).is_err());
        bad = bytes;
        bad[1] = 3;
        assert!(decode_states(&bad, 2).is_err());
        let mut empty = f.items[0].clone();
        empty.stocks[0].remaining = 0;
        empty.use_count = 3;
        empty.request_sequence = 3;
        assert!(empty.validate().is_ok());
    }
    #[test]
    fn use_ack_is_atomic_exact_consumption_and_no_local_optimism() {
        let old = initial();
        let unchanged = old.clone();
        let next = used(&old);
        assert_eq!(old, unchanged);
        next.validate().unwrap();
        assert!(next.confirms(&next.echo, Some(&old)));
        let f = next.frame.as_ref().unwrap();
        assert_eq!(f.vitals[0].hp, 40);
        assert_eq!(f.items[0].stocks[0].remaining, 2);
        let mut bad = next.clone();
        bad.frame.as_mut().unwrap().items[0].request_sequence += 1;
        assert!(!bad.confirms(&next.echo, Some(&old)));
        bad = next.clone();
        bad.echo.item_id = 40011;
        assert!(bad.validate().is_err());
        bad = next.clone();
        bad.frame.as_mut().unwrap().items[0].stocks[0].remaining = 1;
        assert!(bad.validate().is_err());
    }
    #[test]
    fn hp_increase_needs_real_use_and_decline_still_needs_hurt_event() {
        let old = initial();
        let mut fake = old.clone();
        fake.frame.as_mut().unwrap().world.revision += 1;
        fake.frame.as_mut().unwrap().vitals[0].hp += 1;
        assert!(!combat::follows(Some(&old), &fake));
        fake = old.clone();
        fake.frame.as_mut().unwrap().world.revision += 1;
        fake.frame.as_mut().unwrap().vitals[0].hp -= 1;
        assert!(!combat::follows(Some(&old), &fake));
        let f = fake.frame.as_mut().unwrap();
        f.vitals[0].hurt_revision = f.world.revision;
        assert!(combat::follows(Some(&old), &fake));
    }
    #[test]
    fn stale_use_fields_or_new_stock_cannot_be_smuggled_by_denied_sequence() {
        let old = used(&initial()).combat_reply();
        for case in 0..6 {
            let mut bad = old.clone();
            let f = bad.frame.as_mut().unwrap();
            f.world.revision += 1;
            f.items[0].request_sequence += 1;
            match case {
                0 => f.items[0].fixed_recovery += 1,
                1 => f.items[0].stocks[0].initial += 1,
                2 => f.items[0].stocks[0].remaining += 1,
                3 => f.items[0].effect = 194,
                4 => f.items[0].use_revision += 1,
                _ => f.items[0].position.x += 1,
            };
            assert!(!combat::follows(Some(&old), &bad), "case {case}");
        }
        let mut good = old.clone();
        let f = good.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.items[0].request_sequence += 1;
        assert!(combat::follows(Some(&old), &good));
    }
    #[test]
    fn healing_upper_bound_does_not_overflow_and_minimum_is_required_unless_clamped() {
        let mut old = initial();
        {
            let f = old.frame.as_mut().unwrap();
            f.vitals[0].max_hp = i64::MAX;
            f.vitals[0].hp = 1;
            f.items[0].fixed_recovery = i64::MAX - 15;
        }
        let mut next = used(&old);
        {
            let f = next.frame.as_mut().unwrap();
            let s = &mut f.items[0];
            s.restored = i64::MAX - 5;
            s.hp_after = i64::MAX - 4;
            f.vitals[0].hp = s.hp_after;
        }
        next.validate().unwrap();
        assert!(next.confirms(&next.echo, Some(&old)));
        let f = next.frame.as_mut().unwrap();
        f.items[0].restored -= 1;
        f.items[0].hp_after -= 1;
        f.vitals[0].hp -= 1;
        assert!(next.validate().is_err());
        let mut clamped = used(&initial());
        let f = clamped.frame.as_mut().unwrap();
        f.items[0].hp_before = 99;
        f.items[0].hp_after = 100;
        f.items[0].restored = 1;
        f.vitals[0].hp = 100;
        clamped.validate().unwrap();
    }
    #[test]
    fn latest_use_position_exact_but_old_event_survives_following_move() {
        let before = initial();
        let after = used(&before);
        let mut f = after.frame.unwrap();
        f.world.actors[0].position.x -= 1;
        assert!(f.validate().is_err());
        f.world.revision += 1;
        assert!(f.validate().is_ok());
    }
    #[test]
    fn bots_cannot_receive_human_supplies_and_dead_never_revives() {
        let mut f = initial().frame.unwrap();
        f.world.actors[1].kind = super::super::rooms::MemberKind::ServerBot;
        f.world.room.members[1].kind = super::super::rooms::MemberKind::ServerBot;
        f.items[1].fixed_recovery = 1;
        assert!(validate_states(&f).is_err());
        let before = initial();
        let mut dead = before.clone();
        let f = dead.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.vitals[0].hp = 0;
        f.vitals[0].hurt_revision = f.world.revision;
        f.vitals[0].death_revision = f.world.revision;
        dead.validate().unwrap();
        let mut after = used(&dead);
        let f = after.frame.as_mut().unwrap();
        f.items[0].hp_before = 1;
        f.items[0].hp_after = 11;
        f.vitals[0].hp = 11;
        assert!(!combat::follows(Some(&dead), &after.combat_reply()));
    }
    #[test]
    fn multiple_coalesced_uses_bound_net_healing_without_faking_rolls() {
        let before = initial();
        let first = used(&before).combat_reply();
        let second = used(&first);
        assert!(combat::follows(Some(&before), &second.combat_reply()));
        let mut forged = second;
        let f = forged.frame.as_mut().unwrap();
        f.vitals[0].hp = 71;
        f.world.revision += 1;
        assert!(!combat::follows(Some(&before), &forged.combat_reply()));
    }
    #[test]
    fn later_revision_never_grants_hp_above_last_confirmed_restore() {
        let old = initial();
        let mut forged = used(&old).combat_reply();
        let f = forged.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.vitals[0].hp = 50;
        assert!(forged.validate().is_err());
        assert!(!combat::follows(Some(&old), &forged));
    }
    #[test]
    fn latest_actual_restore_replaces_its_theoretical_max_in_coalesced_bound() {
        let old = initial();
        let mut forged = used(&old).combat_reply();
        let f = forged.frame.as_mut().unwrap();
        f.items[0].hp_before = 35;
        f.items[0].hp_after = 45;
        f.vitals[0].hp = 45;
        forged.validate().unwrap();
        assert!(!combat::follows(Some(&old), &forged));
    }
    #[test]
    fn newest_item_event_must_consume_that_item_in_this_transition() {
        let old = used(&initial()).combat_reply();
        let mut forged = used(&old).combat_reply();
        let f = forged.frame.as_mut().unwrap();
        f.items[0].stocks[0].remaining += 1;
        f.items[0].stocks[1].remaining -= 1;
        forged.validate().unwrap();
        assert!(!combat::follows(Some(&old), &forged));
    }
    #[test]
    fn use_cannot_invent_damage_before_or_after_last_heal_without_hurt_revision() {
        let old = initial();
        let mut before = used(&old).combat_reply();
        let f = before.frame.as_mut().unwrap();
        f.items[0].hp_before = 25;
        f.items[0].hp_after = 35;
        f.vitals[0].hp = 35;
        before.validate().unwrap();
        assert!(!combat::follows(Some(&old), &before));
        let mut after = used(&old).combat_reply();
        let f = after.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.vitals[0].hp -= 1;
        assert!(after.validate().is_err());
        let f = after.frame.as_mut().unwrap();
        f.vitals[0].hurt_revision = f.world.revision;
        after.validate().unwrap();
        assert!(combat::follows(Some(&old), &after));
    }
}
