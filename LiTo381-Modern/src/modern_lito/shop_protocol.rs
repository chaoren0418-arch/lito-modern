//! v8 match-local supplies. This wallet and its stacks are not source inventory,
//! saved equipment, real currency, or a permission to use an item in a world.
use super::application_protocol::{self as app, ApplicationError, SelectionContext};
use super::rooms;
use std::collections::BTreeSet;
use std::sync::Arc;
#[cfg(test)]
#[path = "shop_protocol_tests.rs"]
mod tests;

pub const REQUEST_KIND: u16 = 32;
pub const RESULT_KIND: u16 = 33;
pub const MAX_FRAME_BYTES: usize = 256 * 1024;
pub const MAX_ROWS: usize = 256;
pub const MAX_QUANTITY: i64 = 1_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read = 0,
    Buy = 1,
    Sell = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub action: Action,
    pub room_id: u64,
    pub expected_room_revision: i64,
    pub round_id: i64,
    pub expected_revision: i64,
    pub token: [u8; 32],
    pub item_id: i64,
    pub quantity: i64,
}
impl Request {
    pub fn read(
        context: SelectionContext,
        channel_id: u8,
        room: Option<&rooms::RoomSnapshot>,
    ) -> Self {
        Self {
            context,
            channel_id,
            action: Action::Read,
            room_id: room.map_or(0, |r| r.summary.room_id),
            expected_room_revision: room.map_or(0, |r| r.summary.revision),
            round_id: 0,
            expected_revision: 0,
            token: [0; 32],
            item_id: 0,
            quantity: 0,
        }
    }
    pub fn mutation(
        mut self,
        snapshot: &Snapshot,
        action: Action,
        item_id: i64,
        quantity: i64,
    ) -> Self {
        self.action = action;
        self.round_id = snapshot.round_id;
        self.expected_revision = snapshot.revision;
        self.token = snapshot.token;
        self.item_id = item_id;
        self.quantity = quantity;
        self
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || self.room_id > i64::MAX as u64
            || (if self.room_id == 0 {
                self.expected_room_revision != 0
            } else {
                self.expected_room_revision < 1
            })
            || self.expected_revision < 0
            || match self.action {
                Action::Read => {
                    self.round_id != 0
                        || self.expected_revision != 0
                        || self.token != [0; 32]
                        || self.item_id != 0
                        || self.quantity != 0
                }
                _ => {
                    self.round_id < 1
                        || self.item_id < 1
                        || !(1..=MAX_QUANTITY).contains(&self.quantity)
                }
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn room_read(&self) -> rooms::Common {
        rooms::Common::read(self.context, self.channel_id)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok = 0,
    Denied = 1,
    Conflict = 2,
    Busy = 3,
    Unavailable = 4,
    Frozen = 5,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None = 0,
    InsufficientFunds,
    UnsupportedItem,
    NotOwned,
    QuantityLimit,
    Capacity,
    StaleRevision,
    CatalogChanged,
    RoomChanged,
    Frozen,
    Backend,
    Busy,
    TooHeavy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Support {
    Healing = 0,
    Unimplemented = 1,
    UnsupportedTemplate = 2,
    Timed = 3,
    Ammo = 4,
    Material = 5,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Healing {
    pub minimum: i64,
    pub maximum: i64,
    pub effect: i64,
    pub delay_group: i64,
    pub delay_millis: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub item_id: i64,
    pub sort: u16,
    pub support: Support,
    pub buy_price: i64,
    pub sell_price: i64,
    pub inventory_graphic: i64,
    pub ground_graphic: i64,
    pub weight: i64,
    pub name: String,
    pub healing: Option<Healing>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stack {
    pub item_id: i64,
    pub quantity: i64,
    pub unit_paid_price: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimedOffer {
    pub item_id: i64,
    pub kind: super::timed_protocol::Kind,
    pub eligible: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimedCatalog {
    pub owner_class: u8,
    pub offers: Vec<TimedOffer>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub round_id: i64,
    pub revision: i64,
    pub allowance: i64,
    pub balance: i64,
    pub token: [u8; 32],
    pub offers: Vec<Offer>,
    pub stacks: Vec<Stack>,
    pub weight: Option<super::weight_protocol::Snapshot>,
    pub timed: Option<TimedCatalog>,
    pub ammo: Option<Vec<super::ammo_protocol::Facts>>,
    pub materials_enabled: bool,
}
impl Snapshot {
    pub fn timed_offer(&self, id: i64) -> Option<&TimedOffer> {
        self.timed.as_ref()?.offers.iter().find(|o| o.item_id == id)
    }
    pub fn purchasable(&self, id: i64) -> bool {
        self.offer(id).is_some_and(|o| {
            o.support == Support::Healing
                || o.support == Support::Material && self.materials_enabled
                || o.support == Support::Timed && self.timed_offer(id).is_some()
                || o.support == Support::Ammo
                    && self
                        .ammo
                        .as_ref()
                        .is_some_and(|a| a.iter().any(|f| f.item_id == id))
        })
    }
    pub fn offer(&self, id: i64) -> Option<&Offer> {
        self.offers
            .binary_search_by_key(&id, |x| x.item_id)
            .ok()
            .map(|i| &self.offers[i])
    }
    pub fn stack(&self, id: i64) -> Option<&Stack> {
        self.stacks
            .binary_search_by_key(&id, |x| x.item_id)
            .ok()
            .map(|i| &self.stacks[i])
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        let arrows: Vec<_> = self
            .offers
            .iter()
            .filter(|o| o.support == Support::Ammo)
            .map(|o| o.item_id)
            .collect();
        if let Some(ammo) = &self.ammo {
            if ammo.len() > 2 || ammo.len() != arrows.len() {
                return Err(invalid());
            }
            for (id, facts) in arrows.iter().zip(ammo) {
                facts.validate()?;
                if *id != facts.item_id {
                    return Err(invalid());
                }
            }
        } else if !arrows.is_empty() {
            return Err(invalid());
        }
        let offered: Vec<_> = self
            .offers
            .iter()
            .filter(|o| o.support == Support::Timed)
            .map(|o| o.item_id)
            .collect();
        if let Some(timed) = &self.timed {
            if timed.owner_class > 6
                || timed.offers.len() > 4
                || timed.offers.len() != offered.len()
            {
                return Err(invalid());
            }
            for (id, o) in offered.iter().zip(&timed.offers) {
                if *id != o.item_id
                    || o.kind.item_id() != o.item_id
                    || o.eligible != o.kind.eligible(u16::from(timed.owner_class))
                {
                    return Err(invalid());
                }
            }
        } else if !offered.is_empty() {
            return Err(invalid());
        }
        if let Some(weight) = &self.weight {
            weight.validate_stocks(self.stacks.iter().map(|s| (s.item_id, s.quantity)))?;
            if !weight.applicable() {
                return Err(invalid());
            }
        }
        if self.materials_enabled && (self.timed.is_none() || self.weight.is_none()) {
            return Err(invalid());
        }
        if self.round_id < 1
            || self.revision < 0
            || self.allowance < 1
            || !(0..=self.allowance).contains(&self.balance)
            || self.offers.len() > MAX_ROWS
            || self.stacks.len() > MAX_ROWS
        {
            return Err(invalid());
        }
        let mut prior = 0;
        let mut sorts = BTreeSet::new();
        for offer in &self.offers {
            if offer.item_id <= prior
                || !sorts.insert(offer.sort)
                || offer.buy_price < 1
                || offer.sell_price < 1
                || offer.inventory_graphic < 0
                || offer.ground_graphic < 0
                || offer.weight < 0
                || offer.name.len() > 256
                || offer.name.trim().is_empty()
                || offer.name.chars().any(char::is_control)
                || (offer.support == Support::Healing) != offer.healing.is_some()
                || offer.support == Support::Material
                    && (!self.materials_enabled
                        || offer.item_id != 40318
                        || offer.name != "$508"
                        || offer.inventory_graphic != 235
                        || offer.ground_graphic != 773)
            {
                return Err(invalid());
            }
            if let Some(h) = &offer.healing {
                if h.minimum < 1
                    || h.maximum < h.minimum
                    || !(1..=i32::MAX as i64).contains(&h.effect)
                    || h.delay_group < 0
                    || !(1..=10000).contains(&h.delay_millis)
                {
                    return Err(invalid());
                }
            }
            prior = offer.item_id;
        }
        prior = 0;
        for stack in &self.stacks {
            // A fresh catalog may remove an owned offer. Keep the owned stack;
            // never invent a current offer or silently delete its quantity.
            if stack.item_id == 40318 && !self.materials_enabled
                || stack.item_id <= prior
                || !(1..=MAX_QUANTITY).contains(&stack.quantity)
                || stack.unit_paid_price < 1
            {
                return Err(invalid());
            }
            prior = stack.item_id;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: Status,
    pub reason: Reason,
    pub snapshot: Option<Arc<Snapshot>>,
    pub room: Option<rooms::Reply>,
}
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        let present = matches!(self.status, Status::Ok | Status::Denied | Status::Conflict);
        if present != self.snapshot.is_some()
            || present != self.room.is_some()
            || !matches!(
                (self.status, self.reason),
                (Status::Ok, Reason::None)
                    | (
                        Status::Denied,
                        Reason::InsufficientFunds
                            | Reason::UnsupportedItem
                            | Reason::NotOwned
                            | Reason::QuantityLimit
                            | Reason::Capacity
                            | Reason::TooHeavy
                    )
                    | (
                        Status::Conflict,
                        Reason::StaleRevision | Reason::CatalogChanged | Reason::RoomChanged
                    )
                    | (Status::Busy, Reason::Busy)
                    | (Status::Unavailable, Reason::Backend)
                    | (Status::Frozen, Reason::Frozen)
            )
        {
            return Err(invalid());
        }
        if let Some(snapshot) = &self.snapshot {
            snapshot.validate()?;
            if self.reason == Reason::TooHeavy {
                let weight = snapshot.weight.as_ref().ok_or_else(invalid)?;
                let offer = snapshot.offer(self.echo.item_id).ok_or_else(invalid)?;
                if self.echo.action != Action::Buy
                    || snapshot.round_id != self.echo.round_id
                    || snapshot.revision != self.echo.expected_revision
                    || snapshot.token != self.echo.token
                    || !snapshot.purchasable(offer.item_id)
                    || weight.permits_add(self.echo.quantity, offer.weight)
                {
                    return Err(invalid());
                }
            }
            let room = self.room.as_ref().ok_or_else(invalid)?;
            room.validate()?;
            if room.common != self.echo.room_read() || room.status != rooms::Status::Ok {
                return Err(invalid());
            }
            if self.status == Status::Ok {
                if room.joined.as_ref().map_or(0, |r| r.summary.room_id) != self.echo.room_id {
                    return Err(invalid());
                }
                if self.echo.action != Action::Read
                    && (snapshot.round_id != self.echo.round_id
                        || snapshot.token != self.echo.token
                        || self.echo.expected_revision.checked_add(1) != Some(snapshot.revision))
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
    pub fn confirms(
        &self,
        request: &Request,
        prior: Option<&Reply>,
        prior_room: Option<&rooms::Reply>,
    ) -> bool {
        if self.echo != *request || self.validate().is_err() {
            return false;
        }
        if let Some(room) = &self.room {
            if !rooms::follows(prior_room, room) {
                return false;
            }
        }
        let Some(after) = self.snapshot.as_deref() else {
            return true;
        };
        let before = prior
            .filter(|r| {
                r.echo.context == request.context && r.echo.channel_id == request.channel_id
            })
            .and_then(|r| r.snapshot.as_deref());
        if let Some(before) = before {
            if after.round_id < before.round_id
                || after.revision < before.revision
                || after.round_id > before.round_id && after.revision <= before.revision
                || after.revision == before.revision && after != before
            {
                return false;
            }
        }
        if request.action == Action::Read || self.status != Status::Ok {
            return true;
        }
        let Some(before) = before else {
            return false;
        };
        if before.round_id != request.round_id
            || before.revision != request.expected_revision
            || before.token != request.token
            || before.allowance != after.allowance
            || before.offers != after.offers
            || before.timed != after.timed
            || before.ammo != after.ammo
            || before.materials_enabled != after.materials_enabled
            || before.token != after.token
        {
            return false;
        }
        let Some(offer) = before.offer(request.item_id) else {
            return false;
        };
        match (&before.weight, &after.weight) {
            (None, None) => {}
            (Some(old), Some(new)) => {
                if old.source_gear_weight != new.source_gear_weight
                    || old.maximum_bits != new.maximum_bits
                {
                    return false;
                }
                let mut units = old.units.clone();
                match units.binary_search_by_key(&request.item_id, |u| u.item_id) {
                    Ok(i)
                        if request.action == Action::Sell
                            && after.stack(request.item_id).is_none() =>
                    {
                        units.remove(i);
                    }
                    Ok(i)
                        if request.action == Action::Buy
                            && units[i].unit_weight != offer.weight =>
                    {
                        return false;
                    }
                    Err(i) if request.action == Action::Buy => units.insert(
                        i,
                        super::weight_protocol::UnitWeight {
                            item_id: request.item_id,
                            unit_weight: offer.weight,
                        },
                    ),
                    Err(_) => return false,
                    _ => {}
                }
                if new.units != units {
                    return false;
                }
            }
            _ => return false,
        }
        let mut stacks = before.stacks.clone();
        let position = stacks.binary_search_by_key(&request.item_id, |x| x.item_id);
        let balance = match request.action {
            Action::Buy => {
                if !before.purchasable(offer.item_id) {
                    return false;
                }
                match position {
                    Ok(i) => {
                        if stacks[i].unit_paid_price != offer.buy_price {
                            return false;
                        }
                        let Some(q) = stacks[i].quantity.checked_add(request.quantity) else {
                            return false;
                        };
                        stacks[i].quantity = q;
                    }
                    Err(i) => stacks.insert(
                        i,
                        Stack {
                            item_id: request.item_id,
                            quantity: request.quantity,
                            unit_paid_price: offer.buy_price,
                        },
                    ),
                }
                offer
                    .buy_price
                    .checked_mul(request.quantity)
                    .and_then(|cost| before.balance.checked_sub(cost))
            }
            Action::Sell => {
                let Ok(i) = position else {
                    return false;
                };
                let Some(q) = stacks[i]
                    .quantity
                    .checked_sub(request.quantity)
                    .filter(|q| *q >= 0)
                else {
                    return false;
                };
                let refund = stacks[i]
                    .unit_paid_price
                    .min(offer.sell_price)
                    .checked_mul(request.quantity);
                if q == 0 {
                    stacks.remove(i);
                } else {
                    stacks[i].quantity = q;
                }
                refund.and_then(|n| before.balance.checked_add(n))
            }
            Action::Read => unreachable!(),
        };
        balance == Some(after.balance) && stacks == after.stacks
    }
    pub fn notice(&self) -> String {
        if self.reason == Reason::TooHeavy {
            return "購買後將超過比賽負重上限，伺服器未扣款；可先賣回本場補給。".into();
        }
        match self.status {
            Status::Ok => match self.echo.action {
                Action::Read => "本場補給餘額與商品已由伺服器確認。".into(),
                Action::Buy => "購買已由伺服器確認；僅加入本場補給，不寫入來源背包。".into(),
                Action::Sell => "賣回已由伺服器確認；餘額採權威回覆。".into(),
            },
            Status::Busy | Status::Unavailable | Status::Frozen => format!(
                "商店未更新：{:?}；保留前次確認值，不代表零餘額。",
                self.reason
            ),
            _ => format!(
                "伺服器未接受商店操作：{:?}；已更新確認資料，請重新確認。",
                self.reason
            ),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    NotRequested,
    Unsupported,
    Ready {
        confirmed: Arc<Reply>,
        refresh_failure: Option<Status>,
    },
    Unavailable {
        context: SelectionContext,
        channel_id: u8,
        status: Status,
    },
}
impl State {
    /// Discard match/selection data without inventing a capability. A world
    /// transition invalidates a wallet; it cannot enable an unnegotiated shop.
    pub fn negotiated(supported: bool) -> Self {
        if supported {
            Self::NotRequested
        } else {
            Self::Unsupported
        }
    }
    pub fn reply(&self) -> Option<&Reply> {
        match self {
            Self::Ready { confirmed, .. } => Some(confirmed),
            _ => None,
        }
    }
    pub fn snapshot(&self) -> Option<&Snapshot> {
        self.reply().and_then(|r| r.snapshot.as_deref())
    }
    pub fn receive(&mut self, reply: Reply) -> Result<(), ApplicationError> {
        reply.validate()?;
        if let Some(after) = reply.snapshot.as_deref() {
            if let Some(prior) = self
                .reply()
                .filter(|r| {
                    r.echo.context == reply.echo.context
                        && r.echo.channel_id == reply.echo.channel_id
                })
                .and_then(|r| r.snapshot.as_deref())
            {
                if after.round_id < prior.round_id
                    || after.revision < prior.revision
                    || after.round_id > prior.round_id && after.revision <= prior.revision
                    || after.revision == prior.revision && after != prior
                {
                    return Err(invalid());
                }
            }
            *self = Self::Ready {
                confirmed: Arc::new(reply),
                refresh_failure: None,
            };
        } else if let Self::Ready {
            confirmed,
            refresh_failure,
        } = self
        {
            if confirmed.echo.context == reply.echo.context
                && confirmed.echo.channel_id == reply.echo.channel_id
            {
                *refresh_failure = Some(reply.status);
            } else {
                *self = Self::Unavailable {
                    context: reply.echo.context,
                    channel_id: reply.echo.channel_id,
                    status: reply.status,
                };
            }
        } else {
            *self = Self::Unavailable {
                context: reply.echo.context,
                channel_id: reply.echo.channel_id,
                status: reply.status,
            };
        }
        Ok(())
    }
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid match-local shop frame",
    )
}
fn put_request(out: &mut Vec<u8>, r: Request) {
    out.extend(r.context.key.source_id.to_be_bytes());
    out.extend(r.context.key.object_id.to_be_bytes());
    out.extend(r.context.epoch.to_be_bytes());
    out.extend([r.channel_id, 0, 0, 0, r.action as u8, 0, 0, 0]);
    out.extend(r.room_id.to_be_bytes());
    out.extend(r.expected_room_revision.to_be_bytes());
    out.extend(r.round_id.to_be_bytes());
    out.extend(r.expected_revision.to_be_bytes());
    out.extend(r.token);
    out.extend(r.item_id.to_be_bytes());
    out.extend(r.quantity.to_be_bytes());
}
pub fn encode_request(
    id: u32,
    request: Request,
    version: u16,
) -> Result<Vec<u8>, ApplicationError> {
    if !(8..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    request.validate()?;
    let mut out = app::header(id, REQUEST_KIND, 124)?;
    put_request(&mut out, request);
    Ok(out)
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
    fn text(&mut self) -> Result<String, ApplicationError> {
        let n = self.u16()? as usize;
        if n > 256 {
            return Err(invalid());
        }
        std::str::from_utf8(self.take(n)?)
            .map(str::to_owned)
            .map_err(|_| invalid())
    }
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(8..=super::protocol::MAX_VERSION).contains(&version)
        || bytes.len() < 112
        || bytes.len() > MAX_FRAME_BYTES - 16
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
    r.zero(3)?;
    let action = match r.byte()? {
        0 => Action::Read,
        1 => Action::Buy,
        2 => Action::Sell,
        _ => return Err(invalid()),
    };
    r.zero(3)?;
    let echo = Request {
        context,
        channel_id,
        action,
        room_id: r.u64()?,
        expected_room_revision: r.i64()?,
        round_id: r.i64()?,
        expected_revision: r.i64()?,
        token: r.take(32)?.try_into().unwrap(),
        item_id: r.i64()?,
        quantity: r.i64()?,
    };
    let status = match r.byte()? {
        0 => Status::Ok,
        1 => Status::Denied,
        2 => Status::Conflict,
        3 => Status::Busy,
        4 => Status::Unavailable,
        5 => Status::Frozen,
        _ => return Err(invalid()),
    };
    let reason = match r.byte()? {
        0 => Reason::None,
        1 => Reason::InsufficientFunds,
        2 => Reason::UnsupportedItem,
        3 => Reason::NotOwned,
        4 => Reason::QuantityLimit,
        5 => Reason::Capacity,
        6 => Reason::StaleRevision,
        7 => Reason::CatalogChanged,
        8 => Reason::RoomChanged,
        9 => Reason::Frozen,
        10 => Reason::Backend,
        11 => Reason::Busy,
        12 if version >= 9 => Reason::TooHeavy,
        _ => return Err(invalid()),
    };
    let present = r.byte()?;
    r.zero(1)?;
    if present > 1 {
        return Err(invalid());
    }
    let (mut snapshot, room) = if present == 1 {
        let round_id = r.i64()?;
        let revision = r.i64()?;
        let allowance = r.i64()?;
        let balance = r.i64()?;
        let token = r.take(32)?.try_into().unwrap();
        let offers_count = r.u16()? as usize;
        let stacks_count = r.u16()? as usize;
        if offers_count > MAX_ROWS || stacks_count > MAX_ROWS {
            return Err(invalid());
        }
        let mut offers = Vec::with_capacity(offers_count);
        let mut stacks = Vec::with_capacity(stacks_count);
        for _ in 0..offers_count {
            let item_id = r.i64()?;
            let sort = r.u16()?;
            let support = match r.byte()? {
                0 => Support::Healing,
                1 => Support::Unimplemented,
                2 => Support::UnsupportedTemplate,
                3 if version >= 10 => Support::Timed,
                4 if version >= 11 => Support::Ammo,
                5 if version >= 47 => Support::Material,
                _ => return Err(invalid()),
            };
            r.zero(1)?;
            let buy_price = r.i64()?;
            let sell_price = r.i64()?;
            let inventory_graphic = r.i64()?;
            let ground_graphic = r.i64()?;
            let weight = r.i64()?;
            let name = r.text()?;
            let hp = r.byte()?;
            r.zero(3)?;
            let healing = match hp {
                0 => None,
                1 => Some(Healing {
                    minimum: r.i64()?,
                    maximum: r.i64()?,
                    effect: r.i64()?,
                    delay_group: r.i64()?,
                    delay_millis: r.i64()?,
                }),
                _ => return Err(invalid()),
            };
            offers.push(Offer {
                item_id,
                sort,
                support,
                buy_price,
                sell_price,
                inventory_graphic,
                ground_graphic,
                weight,
                name,
                healing,
            });
        }
        for _ in 0..stacks_count {
            stacks.push(Stack {
                item_id: r.i64()?,
                quantity: r.i64()?,
                unit_paid_price: r.i64()?,
            });
        }
        let n = r.u32()? as usize;
        let room = rooms::decode_body_version(r.take(n)?, version)?;
        (
            Some(Arc::new(Snapshot {
                round_id,
                revision,
                allowance,
                balance,
                token,
                offers,
                stacks,
                weight: None,
                timed: None,
                ammo: None,
                materials_enabled: false,
            })),
            Some(room),
        )
    } else {
        (None, None)
    };
    if present == 1 && version >= 9 {
        let (weight, used) = super::weight_protocol::decode_optional(&bytes[r.at..])?;
        r.take(used)?;
        if let Some(weight) = weight {
            Arc::make_mut(snapshot.as_mut().ok_or_else(invalid)?).weight = Some(weight);
        }
    }
    if present == 1 && version >= 10 {
        let enabled = r.byte()?;
        let owner_class = r.byte()?;
        r.zero(2)?;
        let count = r.u16()? as usize;
        r.zero(2)?;
        if enabled > 1
            || count > 4
            || enabled == 0 && (owner_class != 0 || count != 0)
            || owner_class > 6
        {
            return Err(invalid());
        }
        let mut offers = Vec::with_capacity(count);
        for _ in 0..count {
            let item_id = r.i64()?;
            let kind = match r.byte()? {
                0 => super::timed_protocol::Kind::Haste,
                1 => super::timed_protocol::Kind::Brave,
                2 => super::timed_protocol::Kind::ElfBrave,
                3 => super::timed_protocol::Kind::Wisdom,
                _ => return Err(invalid()),
            };
            let eligible = r.byte()?;
            let class_mask = r.byte()?;
            r.zero(1)?;
            let duration = r.i64()?;
            let effect = r.u32()?;
            if eligible > 1
                || class_mask != kind.class_mask()
                || duration != kind.duration_millis()
                || effect != kind.effect() as u32
            {
                return Err(invalid());
            }
            offers.push(TimedOffer {
                item_id,
                kind,
                eligible: eligible == 1,
            });
        }
        if enabled == 1 {
            Arc::make_mut(snapshot.as_mut().ok_or_else(invalid)?).timed = Some(TimedCatalog {
                owner_class,
                offers,
            });
        }
    }
    if present == 1 && version >= 11 {
        let enabled = r.byte()?;
        r.zero(3)?;
        let count = usize::from(r.u16()?);
        r.zero(2)?;
        if enabled > 1 || count > 2 || enabled == 0 && count != 0 {
            return Err(invalid());
        }
        let mut facts = Vec::with_capacity(count);
        for _ in 0..count {
            facts.push(super::ammo_protocol::decode_facts(r.take(32)?)?);
        }
        if enabled == 1 {
            Arc::make_mut(snapshot.as_mut().ok_or_else(invalid)?).ammo = Some(facts);
        }
    }
    if present == 1 && version >= 47 {
        let enabled = r.byte()?;
        r.zero(7)?;
        if enabled > 1 {
            return Err(invalid());
        }
        Arc::make_mut(snapshot.as_mut().ok_or_else(invalid)?).materials_enabled = enabled == 1;
    }
    if r.at != bytes.len() {
        return Err(invalid());
    }
    let reply = Reply {
        echo,
        status,
        reason,
        snapshot,
        room,
    };
    reply.validate()?;
    Ok(reply)
}
