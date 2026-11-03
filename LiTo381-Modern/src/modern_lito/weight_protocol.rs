//! v9 server-confirmed tournament weight. Source weapons/armor only (including
//! unworn gear), plus match-local supplies; not the complete source inventory.
use super::application_protocol::{self as app, ApplicationError};
#[cfg(test)]
#[path = "weight_protocol_tests.rs"]
pub(crate) mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitWeight {
    pub item_id: i64,
    pub unit_weight: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub source_gear_weight: i64,
    pub supplies_weight: i64,
    pub total: i64,
    /// Exact positive finite native binary64. Zero bits only for a server bot.
    pub maximum_bits: u64,
    pub stage: u16,
    pub hit_penalty: u8,
    pub units: Vec<UnitWeight>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorWeight {
    pub entity_id: u64,
    pub weight: Snapshot,
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid authoritative weight",
    )
}
pub fn stack_weight(quantity: i64, unit_weight: i64) -> Result<i64, ApplicationError> {
    if quantity < 0 || unit_weight < 0 {
        return Err(invalid());
    }
    if quantity == 0 || unit_weight == 0 {
        return Ok(0);
    }
    let raw = u128::from(quantity as u64) * u128::from(unit_weight as u64) / 1000;
    i64::try_from(raw.max(1)).map_err(|_| invalid())
}
pub fn penalty(stage: u16) -> u8 {
    match stage {
        81..=120 => 1,
        121..=160 => 3,
        161..=200 => 5,
        _ => 0,
    }
}
pub fn stage(total: i64, maximum: f64) -> Result<u16, ApplicationError> {
    if total < 0 || !maximum.is_finite() || maximum <= 0.0 {
        return Err(invalid());
    }
    let raw = total as f64;
    // Match Java's native binary64 operation order, not rational arithmetic.
    Ok(if raw > maximum {
        240
    } else {
        (raw * 240.0 / maximum).round() as u16
    })
}
impl Snapshot {
    pub fn maximum(&self) -> f64 {
        f64::from_bits(self.maximum_bits)
    }
    pub fn applicable(&self) -> bool {
        self.maximum_bits != 0
    }
    pub fn can_attack(&self) -> bool {
        !self.applicable() || self.stage < 197
    }
    /// Native shop admission: current weight + floor(order / 1000) + 1.
    /// This intentionally differs from the eventual merged stack's weight.
    pub fn permits_add(&self, quantity: i64, unit_weight: i64) -> bool {
        if self.validate().is_err() || !self.applicable() || quantity < 1 || unit_weight < 0 {
            return false;
        }
        let order = u128::from(quantity as u64) * u128::from(unit_weight as u64) / 1000;
        let projected = i64::try_from(order)
            .ok()
            .and_then(|n| n.checked_add(1))
            .and_then(|n| self.total.checked_add(n));
        projected.is_some_and(|n| stage(n, self.maximum()).is_ok_and(|s| s < 240))
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if self.source_gear_weight < 0
            || self.supplies_weight < 0
            || self.total < 0
            || self.source_gear_weight.checked_add(self.supplies_weight) != Some(self.total)
            || self.units.len() > 256
        {
            return Err(invalid());
        }
        if !self.applicable() {
            return if self.source_gear_weight == 0
                && self.supplies_weight == 0
                && self.total == 0
                && self.stage == 0
                && self.hit_penalty == 0
                && self.units.is_empty()
            {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        if stage(self.total, self.maximum())? != self.stage
            || penalty(self.stage) != self.hit_penalty
        {
            return Err(invalid());
        }
        let mut previous = 0;
        for unit in &self.units {
            if unit.item_id <= previous || unit.unit_weight < 0 {
                return Err(invalid());
            }
            previous = unit.item_id;
        }
        Ok(())
    }
    pub fn validate_stocks(
        &self,
        stocks: impl IntoIterator<Item = (i64, i64)>,
    ) -> Result<(), ApplicationError> {
        self.validate()?;
        let mut stocks = stocks.into_iter();
        let mut total = 0i64;
        for unit in &self.units {
            let (id, quantity) = stocks.next().ok_or_else(invalid)?;
            if id != unit.item_id {
                return Err(invalid());
            }
            total = total
                .checked_add(stack_weight(quantity, unit.unit_weight)?)
                .ok_or_else(invalid)?;
        }
        if stocks.next().is_some() || total != self.supplies_weight {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn notice(&self) -> String {
        if !self.applicable() {
            return "伺服器假人：負重不適用。".into();
        }
        format!(
            "比賽負重 {}／{:.3}（{:.1}% · 階段 {}/240）\r\n來源有效武器／防具 {}（含未裝備；排除已到期；不含 ETC／金幣）＋本場補給 {}。{}",
            self.total,
            self.maximum(),
            f64::from(self.stage) * 100.0 / 240.0,
            self.stage,
            self.source_gear_weight,
            self.supplies_weight,
            if self.can_attack() {
                "可攻擊／施法；數值由伺服器確認。"
            } else {
                "過重：不能普攻／施法；仍可移動及喝藥減重。"
            }
        )
    }
}
pub fn validate_world(frame: &super::combat_protocol::Frame) -> Result<(), ApplicationError> {
    if frame.weights.is_empty() {
        return Ok(());
    }
    if frame.weights.len() != frame.world.actors.len() || frame.items.len() != frame.weights.len() {
        return Err(invalid());
    }
    for ((row, actor), items) in frame
        .weights
        .iter()
        .zip(&frame.world.actors)
        .zip(&frame.items)
    {
        if row.entity_id != actor.entity_id
            || row.weight.applicable() != actor.identity().character().is_some()
        {
            return Err(invalid());
        }
        let mut stocks: Vec<_> = items
            .stocks
            .iter()
            .map(|s| (s.item_id, s.remaining))
            .collect();
        if let Some(ammo) = frame.ammo(actor.entity_id) {
            stocks.extend(ammo.stocks.iter().map(|s| (s.facts.item_id, s.remaining)));
        }
        stocks.sort_unstable_by_key(|s| s.0);
        if stocks.len() > 9 || stocks.windows(2).any(|s| s[0].0 == s[1].0) {
            return Err(invalid());
        }
        row.weight.validate_stocks(stocks)?;
    }
    Ok(())
}
pub fn follows(a: &super::combat_protocol::Frame, b: &super::combat_protocol::Frame) -> bool {
    a.weights.len() == b.weights.len()
        && a.weights.iter().zip(&b.weights).all(|(a, b)| {
            a.entity_id == b.entity_id
                && a.weight.source_gear_weight == b.weight.source_gear_weight
                && a.weight.maximum_bits == b.weight.maximum_bits
                && a.weight.units == b.weight.units
                && b.weight.supplies_weight <= a.weight.supplies_weight
        })
}
pub(crate) fn negotiated(response: &app::Response, enabled: bool) -> bool {
    let matches = |f: &super::combat_protocol::Frame| {
        f.world.feature_presence_matches(
            super::world_protocol::MatchDescriptor::WEIGHT,
            !f.weights.is_empty(),
            enabled,
        )
    };
    let weight_reason = match response {
        app::Response::Shop(r) => r.reason == super::shop_protocol::Reason::TooHeavy,
        app::Response::Combat(r) => r.reason == super::combat_protocol::Reason::TooHeavy,
        app::Response::Siege(r) => r.reason == super::siege_protocol::Reason::TooHeavy,
        app::Response::Cast(r) => r.reason == super::spell_protocol::Reason::TooHeavy,
        _ => false,
    };
    if weight_reason && !enabled {
        return false;
    }
    match response {
        app::Response::Shop(r) => r
            .snapshot
            .as_ref()
            .is_none_or(|s| s.weight.is_some() == enabled),
        app::Response::Equipment(r) => r
            .snapshot
            .as_ref()
            .is_none_or(|s| s.weight.is_some() == enabled),
        app::Response::Combat(r) => r.frame.as_ref().is_none_or(matches),
        app::Response::Siege(r) => r.frame.as_ref().is_none_or(matches),
        app::Response::Cast(r) => r.frame.as_ref().is_none_or(matches),
        app::Response::Item(r) => r.frame.as_ref().is_none_or(matches),
        app::Response::Ammo(r) => r.frame.as_ref().is_none_or(matches),
        _ => true,
    }
}
pub(crate) fn decode(bytes: &[u8]) -> Result<(Snapshot, usize), ApplicationError> {
    if bytes.len() < 40 || bytes[35] != 0 || bytes[38..40] != [0, 0] {
        return Err(invalid());
    }
    let n = u16::from_be_bytes(bytes[36..38].try_into().unwrap()) as usize;
    let used = 40 + n * 16;
    if n > 256 || bytes.len() < used {
        return Err(invalid());
    }
    let i64_at = |at: usize| i64::from_be_bytes(bytes[at..at + 8].try_into().unwrap());
    let mut units = Vec::with_capacity(n);
    for i in 0..n {
        units.push(UnitWeight {
            item_id: i64_at(40 + i * 16),
            unit_weight: i64_at(48 + i * 16),
        });
    }
    let result = Snapshot {
        source_gear_weight: i64_at(0),
        supplies_weight: i64_at(8),
        total: i64_at(16),
        maximum_bits: u64::from_be_bytes(bytes[24..32].try_into().unwrap()),
        stage: u16::from_be_bytes(bytes[32..34].try_into().unwrap()),
        hit_penalty: bytes[34],
        units,
    };
    result.validate()?;
    Ok((result, used))
}
pub(crate) fn decode_optional(bytes: &[u8]) -> Result<(Option<Snapshot>, usize), ApplicationError> {
    if bytes.len() < 8 || bytes[1..8].iter().any(|b| *b != 0) {
        return Err(invalid());
    }
    match bytes[0] {
        0 => Ok((None, 8)),
        1 => {
            let (s, n) = decode(&bytes[8..])?;
            Ok((Some(s), n + 8))
        }
        _ => Err(invalid()),
    }
}
pub(crate) fn decode_actors(
    bytes: &[u8],
    actors: usize,
    sparse: bool,
) -> Result<(Vec<ActorWeight>, usize), ApplicationError> {
    if bytes.len() < 8 || bytes[2..8].iter().any(|b| *b != 0) {
        return Err(invalid());
    }
    let count = u16::from_be_bytes(bytes[..2].try_into().unwrap()) as usize;
    if count > 100 || count != 0 && count != actors || bytes.len() - 8 < count * 48 {
        return Err(invalid());
    }
    let mut rows = Vec::with_capacity(count);
    let mut at = 8;
    let mut previous = 0;
    for i in 0..count {
        if bytes.len() - at < 8 {
            return Err(invalid());
        }
        let entity_id = u64::from_be_bytes(bytes[at..at + 8].try_into().unwrap());
        if entity_id <= previous || entity_id > 100 || !sparse && entity_id != i as u64 + 1 {
            return Err(invalid());
        }
        let (weight, n) = decode(&bytes[at + 8..])?;
        rows.push(ActorWeight { entity_id, weight });
        previous = entity_id;
        at += 8 + n;
    }
    Ok((rows, at))
}
