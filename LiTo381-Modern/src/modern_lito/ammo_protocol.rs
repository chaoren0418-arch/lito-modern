//! v11 server-owned ammunition. Arrow selection and shots are independent of
//! the seven consumable USE counters; a miss is still an accepted paid shot.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    combat_protocol as combat,
    world_map::WorldCell,
    world_protocol as world,
};
pub const REQUEST_KIND: u16 = 36;
pub const RESULT_KIND: u16 = 37;
pub const UI_ITEMS: [i64; 2] = [40744, 40746];
pub const MAX_STOCKS: usize = 2;
pub fn supported(id: i64) -> bool {
    UI_ITEMS.contains(&id)
}
pub fn bow(kind: u8) -> bool {
    matches!(kind, 4 | 13)
}
fn positive(n: u64) -> bool {
    n > 0 && n <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid authoritative ammunition",
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts {
    pub item_id: i64,
    pub small: i64,
    pub large: i64,
    pub raw_arrow_sprite: i32,
    pub consume: bool,
    pub material: u8,
    pub blessing: u8,
}
impl Facts {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !supported(self.item_id)
            || self.small < 1
            || self.large < 1
            || !matches!(self.raw_arrow_sprite, -1 | 66)
            || !self.consume
            || self.material != if self.item_id == 40744 { 0 } else { 1 }
            || self.blessing != 0
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stock {
    pub facts: Facts,
    pub initial: i64,
    pub remaining: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub entity_id: u64,
    pub request_sequence: u64,
    pub revision: i64,
    pub selection_revision: i64,
    pub selected_item_id: i64,
    pub shot_count: u64,
    pub shot_revision: i64,
    pub shot_item_id: i64,
    pub attack_sequence: u64,
    pub last_target_entity_id: u64,
    pub from: WorldCell,
    pub to: WorldCell,
    pub projectile_sprite: i32,
    pub hit: bool,
    pub stocks: Vec<Stock>,
}
impl State {
    pub fn initial(entity_id: u64, stocks: Vec<Stock>) -> Self {
        Self {
            entity_id,
            request_sequence: 0,
            revision: 0,
            selection_revision: 0,
            selected_item_id: stocks.first().map_or(0, |s| s.facts.item_id),
            shot_count: 0,
            shot_revision: 0,
            shot_item_id: 0,
            attack_sequence: 0,
            last_target_entity_id: 0,
            from: WorldCell { x: 0, y: 0 },
            to: WorldCell { x: 0, y: 0 },
            projectile_sprite: 0,
            hit: false,
            stocks,
        }
    }
    pub fn stock(&self, item_id: i64) -> Option<&Stock> {
        self.stocks.iter().find(|s| s.facts.item_id == item_id)
    }
    pub fn available(&self) -> bool {
        self.stock(self.selected_item_id)
            .is_some_and(|s| s.remaining > 0)
    }
    fn same_shot(&self, other: &Self) -> bool {
        self.shot_count == other.shot_count
            && self.shot_revision == other.shot_revision
            && self.shot_item_id == other.shot_item_id
            && self.attack_sequence == other.attack_sequence
            && self.last_target_entity_id == other.last_target_entity_id
            && self.from == other.from
            && self.to == other.to
            && self.projectile_sprite == other.projectile_sprite
            && self.hit == other.hit
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !positive(self.entity_id)
            || self.request_sequence > i64::MAX as u64
            || self.shot_count > i64::MAX as u64
            || self.attack_sequence > i64::MAX as u64
            || self.revision < 0
            || self.selection_revision < 0
            || self.shot_revision < 0
            || self.selection_revision > self.revision
            || self.shot_revision > self.revision
            || self.stocks.len() > MAX_STOCKS
        {
            return Err(invalid());
        }
        let mut previous = 0;
        let mut spent = 0u64;
        for s in &self.stocks {
            s.facts.validate()?;
            if s.facts.item_id <= previous
                || !(1..=1_000_000).contains(&s.initial)
                || !(0..=s.initial).contains(&s.remaining)
            {
                return Err(invalid());
            }
            spent = spent
                .checked_add((s.initial - s.remaining) as u64)
                .ok_or_else(invalid)?;
            previous = s.facts.item_id;
        }
        if spent != self.shot_count
            || self.shot_count > self.attack_sequence
            || (self.stocks.is_empty() && self.selected_item_id != 0)
            || (!self.stocks.is_empty() && self.stock(self.selected_item_id).is_none())
            || self.selection_revision == 0
                && self.selected_item_id != self.stocks.first().map_or(0, |s| s.facts.item_id)
            || self.selection_revision == 0
                && self
                    .stocks
                    .iter()
                    .any(|s| s.facts.item_id != self.selected_item_id && s.remaining != s.initial)
            || self.selection_revision > 0 && self.request_sequence == 0
            || (self.revision == 0) != (self.request_sequence == 0 && self.shot_count == 0)
        {
            return Err(invalid());
        }
        if self.shot_count == 0 {
            if !self.same_shot(&Self::initial(self.entity_id, vec![])) {
                return Err(invalid());
            }
        } else if self.shot_revision == 0
            || !positive(self.attack_sequence)
            || self.shot_revision == self.selection_revision
            || self.shot_revision > self.selection_revision
                && self.shot_item_id != self.selected_item_id
            || !positive(self.last_target_entity_id)
            || self.last_target_entity_id == self.entity_id
            || self.projectile_sprite != 66
            || self.from == self.to
            || !self
                .stock(self.shot_item_id)
                .is_some_and(|s| s.remaining < s.initial)
        {
            return Err(invalid());
        }
        Ok(())
    }
}
pub fn validate_frame(frame: &combat::Frame) -> Result<(), ApplicationError> {
    if frame.ammo.is_empty() {
        return Ok(());
    }
    if frame.ammo.len() != frame.world.actors.len() || frame.vitals.len() != frame.ammo.len() {
        return Err(invalid());
    }
    for ((s, actor), vital) in frame
        .ammo
        .iter()
        .zip(&frame.world.actors)
        .zip(&frame.vitals)
    {
        s.validate()?;
        if s.entity_id != actor.entity_id
            || s.revision > frame.world.revision
            || s.attack_sequence > vital.attack_sequence
            || actor.identity().character().is_none()
                && *s != State::initial(actor.entity_id, vec![])
            || matches!(
                frame.world.phase,
                world::Phase::Loading | world::Phase::Countdown
            ) && s.revision != 0
            || s.shot_count > 0
                && (!bow(vital.weapon_type)
                    || s.shot_revision != vital.attack_revision
                    || s.last_target_entity_id != vital.last_target_entity_id
                    || super::honor_protocol::target(frame, s.last_target_entity_id).is_none()
                    || !frame.world.manifest.contains(s.from)
                    || !frame.world.manifest.contains(s.to)
                    || s.shot_revision == frame.world.revision
                        && (actor.position != s.from
                            || super::honor_protocol::target_position(
                                frame,
                                s.last_target_entity_id,
                            ) != Some(s.to)
                            || !s.hit
                                && super::honor_protocol::target_hurt_revision(
                                    frame,
                                    s.last_target_entity_id,
                                ) == Some(s.shot_revision)))
            || bow(vital.weapon_type) && vital.attack_revision > 0 && s.shot_count == 0
            || frame.world.completion.as_ref().is_some_and(|c| {
                c.event
                    .engine_revision
                    .checked_add(1)
                    .is_none_or(|r| s.revision > r)
            })
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub fn follows(a: &combat::Frame, b: &combat::Frame) -> bool {
    if a.ammo.len() != b.ammo.len() {
        return false;
    }
    a.ammo
        .iter()
        .zip(&b.ammo)
        .zip(a.vitals.iter().zip(&b.vitals))
        .all(|((old, new), (ov, nv))| {
            if old.entity_id != new.entity_id
                || new.revision < old.revision
                || new.request_sequence < old.request_sequence
                || new.selection_revision < old.selection_revision
                || new.shot_count < old.shot_count
                || new.shot_revision < old.shot_revision
                || old.stocks.len() != new.stocks.len()
                || !old.stocks.iter().zip(&new.stocks).all(|(a, b)| {
                    a.facts == b.facts && a.initial == b.initial && b.remaining <= a.remaining
                })
                || new.selection_revision == old.selection_revision
                    && new.selected_item_id != old.selected_item_id
                || new.selection_revision == old.selection_revision
                    && old.stocks.iter().zip(&new.stocks).any(|(a, b)| {
                        a.facts.item_id != old.selected_item_id && a.remaining != b.remaining
                    })
                || new.selection_revision > old.selection_revision
                    && (new.request_sequence == old.request_sequence
                        || new.selection_revision <= a.world.revision)
            {
                return false;
            }
            if new.revision == old.revision {
                return new == old;
            }
            if new.revision <= a.world.revision
                || new.request_sequence == old.request_sequence && new.shot_count == old.shot_count
            {
                return false;
            }
            if new.shot_count == old.shot_count {
                new.same_shot(old) && new.stocks == old.stocks
            } else {
                new.shot_revision > a.world.revision
                    && new.attack_sequence > old.attack_sequence
                    && nv
                        .attack_sequence
                        .checked_sub(ov.attack_sequence)
                        .is_some_and(|d| new.shot_count - old.shot_count <= d)
                    && old
                        .stock(new.shot_item_id)
                        .zip(new.stock(new.shot_item_id))
                        .is_some_and(|(a, b)| b.remaining < a.remaining)
            }
        })
}
pub fn confirms_attack(prior: Option<&combat::Reply>, next: &combat::Reply) -> bool {
    if next.echo.action != combat::Action::Attack {
        return true;
    }
    let Some(f) = &next.frame else {
        return true;
    };
    if f.ammo.is_empty() {
        return true;
    }
    let Some(actor) = f
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(next.echo.context.key))
    else {
        return false;
    };
    let Some(after) = f.ammo(actor.entity_id) else {
        return false;
    };
    let Some(before) = prior
        .and_then(|r| r.frame.as_ref())
        .and_then(|f| f.ammo(actor.entity_id))
    else {
        return false;
    };
    if next.status != combat::Status::Ok {
        return after == before;
    }
    if !f.vital(actor.entity_id).is_some_and(|v| bow(v.weapon_type)) {
        return after == before;
    }
    before.shot_count.checked_add(1) == Some(after.shot_count)
        && after.shot_revision == f.world.revision
        && after.revision == f.world.revision
        && after.attack_sequence == next.echo.sequence
        && after.last_target_entity_id == next.echo.target_entity_id
        && after.request_sequence == before.request_sequence
        && after.selection_revision == before.selection_revision
        && after.selected_item_id == before.selected_item_id
        && after.shot_item_id == before.selected_item_id
        && before.stocks.iter().zip(&after.stocks).all(|(a, b)| {
            a.remaining
                .checked_sub(i64::from(a.facts.item_id == after.shot_item_id))
                == Some(b.remaining)
        })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Select,
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
    Unsupported,
    Numeric,
}
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
                Action::Select => !positive(self.sequence) || self.item_id <= 0,
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
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
        let actor = f
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(self.echo.context.key))
            .ok_or_else(invalid)?;
        let state = f.ammo(actor.entity_id).ok_or_else(invalid)?;
        if f.world.match_id != self.echo.match_id || f.world.generation != self.echo.generation {
            return Err(invalid());
        }
        if self.echo.action == Action::Read
            && (self.status != Status::Ok || self.reason != Reason::None)
        {
            return Err(invalid());
        }
        if self.reason == Reason::NotActive
            && (self.echo.action != Action::Select
                || f.world.phase == world::Phase::Active
                || self.status != Status::Denied)
        {
            return Err(invalid());
        }
        if self.echo.action == Action::Select {
            if self.status == Status::Ok
                && (f.world.phase != world::Phase::Active
                    || f.vital(actor.entity_id).is_none_or(|v| v.hp == 0)
                    || state.request_sequence != self.echo.sequence
                    || state.revision != f.world.revision
                    || state.selection_revision != f.world.revision
                    || state.selected_item_id != self.echo.item_id
                    || !state.available())
            {
                return Err(invalid());
            }
            if f.world.phase == world::Phase::Active
                && (state.request_sequence != self.echo.sequence
                    || state.revision != f.world.revision)
            {
                return Err(invalid());
            }
            if f.world.phase != world::Phase::Active
                && (self.status != Status::Denied || self.reason != Reason::NotActive)
            {
                return Err(invalid());
            }
        }
        if matches!(
            self.reason,
            Reason::Dead | Reason::Empty | Reason::Unsupported | Reason::Numeric
        ) {
            if self.echo.action != Action::Select
                || self.status != Status::Denied
                || f.world.phase != world::Phase::Active
            {
                return Err(invalid());
            }
            if state.selection_revision == state.revision {
                return Err(invalid());
            }
            let alive = f.vital(actor.entity_id).is_some_and(|v| v.hp > 0);
            if match self.reason {
                Reason::Dead => alive,
                Reason::Empty => {
                    !alive
                        || !supported(self.echo.item_id)
                        || state
                            .stock(self.echo.item_id)
                            .is_some_and(|s| s.remaining > 0)
                }
                Reason::Unsupported => !alive || supported(self.echo.item_id),
                Reason::Numeric => !alive,
                _ => false,
            } {
                return Err(invalid());
            }
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
        if request.action == Action::Read || self.frame.is_none() {
            return true;
        }
        let f = self.frame.as_ref().unwrap();
        let Some(actor) = f
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(request.context.key))
        else {
            return false;
        };
        let Some(old) = prior
            .and_then(|r| r.frame.as_ref())
            .and_then(|f| f.ammo(actor.entity_id))
        else {
            return false;
        };
        let Some(new) = f.ammo(actor.entity_id) else {
            return false;
        };
        if f.world.phase != world::Phase::Active {
            return old == new && self.status == Status::Denied && self.reason == Reason::NotActive;
        }
        old.same_shot(new)
            && old.stocks == new.stocks
            && old.request_sequence.checked_add(1) == Some(request.sequence)
            && if self.status == Status::Ok {
                true
            } else {
                new.selected_item_id == old.selected_item_id
                    && new.selection_revision == old.selection_revision
                    && new.request_sequence == request.sequence
            }
    }
}
pub(crate) fn negotiated(response: &app::Response, enabled: bool) -> bool {
    let matches = |f: &combat::Frame| {
        f.world.feature_presence_matches(
            super::world_protocol::MatchDescriptor::AMMO,
            !f.ammo.is_empty(),
            enabled,
        )
    };
    match response {
        app::Response::Shop(r) => r
            .snapshot
            .as_ref()
            .is_none_or(|s| s.ammo.is_some() == enabled),
        app::Response::Combat(r) => {
            r.frame.as_ref().is_none_or(matches) && (r.reason != combat::Reason::NoAmmo || enabled)
        }
        app::Response::Cast(r) => r.frame.as_ref().is_none_or(matches),
        app::Response::Siege(r) => {
            r.frame.as_ref().is_none_or(matches)
                && (r.reason != super::siege_protocol::Reason::NoAmmo || enabled)
        }
        app::Response::Item(r) => r.frame.as_ref().is_none_or(matches),
        app::Response::Ammo(r) => {
            enabled
                && r.frame
                    .as_ref()
                    .is_none_or(|f| !f.ammo.is_empty() && matches(f))
        }
        _ => true,
    }
}
pub fn encode_request(id: u32, r: Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    if !(11..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    r.validate()?;
    let mut out = app::header(id, REQUEST_KIND, 72)?;
    out.extend(r.context.key.source_id.to_be_bytes());
    out.extend(r.context.key.object_id.to_be_bytes());
    out.extend(r.context.epoch.to_be_bytes());
    out.extend([r.channel_id, r.action as u8, 0, 0]);
    out.extend(r.match_id.to_be_bytes());
    out.extend(r.generation.to_be_bytes());
    out.extend(r.sequence.to_be_bytes());
    out.extend(r.item_id.to_be_bytes());
    Ok(out)
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.at.checked_add(n).ok_or_else(invalid)?;
        let b = self.bytes.get(self.at..end).ok_or_else(invalid)?;
        self.at = end;
        Ok(b)
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
    fn bool(&mut self) -> Result<bool, ApplicationError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid()),
        }
    }
}
pub(crate) fn decode_facts(bytes: &[u8]) -> Result<Facts, ApplicationError> {
    let mut r = Reader { bytes, at: 0 };
    let facts = Facts {
        item_id: r.i64()?,
        small: r.i64()?,
        large: r.i64()?,
        raw_arrow_sprite: r.u32()? as i32,
        consume: r.bool()?,
        material: r.byte()?,
        blessing: r.byte()?,
    };
    r.zero(1)?;
    if r.at != bytes.len() {
        return Err(invalid());
    }
    facts.validate()?;
    Ok(facts)
}
pub(crate) fn decode_states(
    bytes: &[u8],
    actors: usize,
) -> Result<(Vec<State>, usize), ApplicationError> {
    let mut r = Reader { bytes, at: 0 };
    let count = usize::from(r.u16()?);
    r.zero(6)?;
    if count > 100 || count != 0 && count != actors {
        return Err(invalid());
    }
    let mut states = Vec::with_capacity(count);
    for index in 0..count {
        let mut state = State {
            entity_id: r.u64()?,
            request_sequence: r.u64()?,
            revision: r.i64()?,
            selection_revision: r.i64()?,
            selected_item_id: r.i64()?,
            shot_count: r.u64()?,
            shot_revision: r.i64()?,
            shot_item_id: r.i64()?,
            attack_sequence: r.u64()?,
            last_target_entity_id: r.u64()?,
            from: WorldCell {
                x: r.u32()? as i32,
                y: r.u32()? as i32,
            },
            to: WorldCell {
                x: r.u32()? as i32,
                y: r.u32()? as i32,
            },
            projectile_sprite: r.u32()? as i32,
            hit: r.bool()?,
            stocks: vec![],
        };
        let stocks = usize::from(r.byte()?);
        r.zero(2)?;
        if stocks > MAX_STOCKS || state.entity_id != index as u64 + 1 {
            return Err(invalid());
        }
        for _ in 0..stocks {
            let id = r.i64()?;
            let initial = r.i64()?;
            let remaining = r.i64()?;
            let facts = Facts {
                item_id: id,
                small: r.i64()?,
                large: r.i64()?,
                raw_arrow_sprite: r.u32()? as i32,
                consume: r.bool()?,
                material: r.byte()?,
                blessing: r.byte()?,
            };
            r.zero(1)?;
            state.stocks.push(Stock {
                facts,
                initial,
                remaining,
            });
        }
        state.validate()?;
        states.push(state);
    }
    Ok((states, r.at))
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(11..=super::protocol::MAX_VERSION).contains(&version)
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
        1 => Action::Select,
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
        4 => Reason::Unsupported,
        5 => Reason::Numeric,
        _ => return Err(invalid()),
    };
    let present = r.bool()?;
    r.zero(1)?;
    let frame = if present {
        let (f, n) = combat::decode_frame_version(&bytes[r.at..], context.key, version)?;
        r.take(n)?;
        Some(f)
    } else {
        None
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
#[path = "ammo_protocol_tests.rs"]
pub(crate) mod tests;
