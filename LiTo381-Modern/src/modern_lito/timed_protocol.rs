//! v10 authority-observed timed supplies. Client clocks estimate display only;
//! only a later acknowledged state may change effective movement/attack speed.
use super::{
    application_protocol::{self as app, ApplicationError},
    combat_protocol as combat, world_protocol as world,
};

pub const MAX_OBSERVED_NANOS: i64 = 30 * 60 * 1_000_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Haste = 0,
    Brave = 1,
    ElfBrave = 2,
    Wisdom = 3,
}
impl Kind {
    pub const ALL: [Self; 4] = [Self::Haste, Self::Brave, Self::ElfBrave, Self::Wisdom];
    pub fn item_id(self) -> i64 {
        match self {
            Self::Haste => 40013,
            Self::Brave => 40014,
            Self::ElfBrave => 40068,
            Self::Wisdom => 40016,
        }
    }
    pub fn from_item(id: i64) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.item_id() == id)
    }
    pub fn duration_millis(self) -> i64 {
        if self == Self::ElfBrave {
            480000
        } else {
            300000
        }
    }
    pub fn effect(self) -> i32 {
        match self {
            Self::Haste => 191,
            Self::Brave | Self::ElfBrave => 751,
            Self::Wisdom => 750,
        }
    }
    pub fn class_mask(self) -> u8 {
        match self {
            Self::Haste | Self::Brave => 127,
            Self::ElfBrave => 4,
            Self::Wisdom => 8,
        }
    }
    pub fn eligible(self, class: u16) -> bool {
        class <= 6 && self.class_mask() & (1u8 << class) != 0
    }
    pub fn icon(self) -> i64 {
        match self {
            Self::Haste => 77,
            Self::Brave => 79,
            Self::ElfBrave => 130,
            Self::Wisdom => 1529,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Haste => "加速藥水",
            Self::Brave => "勇敢藥水",
            Self::ElfBrave => "精靈餅乾",
            Self::Wisdom => "慎重藥水",
        }
    }
    pub fn description_id(self) -> u32 {
        match self {
            Self::Haste => 234,
            Self::Brave => 943,
            Self::ElfBrave => 110,
            Self::Wisdom => 944,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    Move,
    Attack,
    Spell,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub entity_id: u64,
    pub revision: i64,
    pub observed_nanos: i64,
    pub haste_until_nanos: i64,
    pub brave_until_nanos: i64,
    pub wisdom_until_nanos: i64,
    pub brave_kind: u8,
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid timed supply state")
}
impl State {
    pub fn initial(entity_id: u64) -> Self {
        Self {
            entity_id,
            revision: 0,
            observed_nanos: 0,
            haste_until_nanos: 0,
            brave_until_nanos: 0,
            wisdom_until_nanos: 0,
            brave_kind: 0,
        }
    }
    pub fn haste(&self) -> bool {
        self.haste_until_nanos > self.observed_nanos
    }
    pub fn brave(&self) -> bool {
        self.brave_until_nanos > self.observed_nanos
    }
    pub fn wisdom(&self) -> bool {
        self.wisdom_until_nanos > self.observed_nanos
    }
    pub fn spell_power_bonus(&self) -> i64 {
        if self.wisdom() {
            2
        } else {
            0
        }
    }
    pub fn active(&self, kind: Kind) -> bool {
        match kind {
            Kind::Haste => self.haste(),
            Kind::Brave => self.brave() && self.brave_kind == 1,
            Kind::ElfBrave => self.brave() && self.brave_kind == 3,
            Kind::Wisdom => self.wisdom(),
        }
    }
    pub fn deadline(&self, kind: Kind) -> i64 {
        match kind {
            Kind::Haste => self.haste_until_nanos,
            Kind::Brave | Kind::ElfBrave => self.brave_until_nanos,
            Kind::Wisdom => self.wisdom_until_nanos,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.validate_haste(300_000_000_000)
    }
    /// Shape only; canonical skill receipt/source checks must also succeed.
    /// Legacy decoding always uses validate(), never this extended bound.
    pub(crate) fn validate_greater_haste(&self) -> Result<(), ApplicationError> {
        if self.haste_until_nanos == 0 {
            return Err(invalid());
        }
        self.validate_haste(2_400_000_000_000)
    }
    fn validate_haste(&self, haste_maximum: i64) -> Result<(), ApplicationError> {
        if self.entity_id == 0
            || self.entity_id > i64::MAX as u64
            || self.revision < 0
            || !(0..=MAX_OBSERVED_NANOS).contains(&self.observed_nanos)
            || !matches!(self.brave_kind, 0 | 1 | 3)
            || (self.brave_until_nanos == 0) != (self.brave_kind == 0)
        {
            return Err(invalid());
        }
        for (deadline, duration) in [
            (self.haste_until_nanos, haste_maximum),
            (
                self.brave_until_nanos,
                if self.brave_kind == 3 {
                    480000000000
                } else {
                    300000000000
                },
            ),
            (self.wisdom_until_nanos, 300000000000),
        ] {
            if deadline != 0
                && (deadline <= self.observed_nanos || deadline - self.observed_nanos > duration)
            {
                return Err(invalid());
            }
        }
        if self.revision == 0 && *self != Self::initial(self.entity_id) {
            return Err(invalid());
        }
        Ok(())
    }
    /// Retain every native binary64 multiply/truncate stage. Source intervals
    /// stay immutable in WORLD/Vital; visual/projectile clocks are not changed.
    pub fn effective_nanos(&self, base_nanos: u64, pace: Pace) -> u64 {
        if !self.haste() && !self.brave() {
            return base_nanos;
        }
        let mut millis = (base_nanos / 1_000_000).max(1);
        if self.haste() {
            millis = (millis as f64 * 0.755) as u64;
        }
        if self.brave() {
            millis = (millis as f64 * 0.755) as u64;
        }
        if self.brave() && self.brave_kind == 3 {
            millis = (millis as f64 * 0.855) as u64;
            if pace == Pace::Attack {
                millis = (millis as f64 * 0.9) as u64;
            }
        }
        millis.max(1) * 1_000_000
    }
    pub fn notice(&self) -> String {
        self.notice_estimate(std::time::Duration::ZERO)
    }
    pub fn notice_estimate(&self, since_receipt: std::time::Duration) -> String {
        let mut effects = Vec::new();
        for k in Kind::ALL {
            if self.active(k) {
                let left = std::time::Duration::from_nanos(
                    (self.deadline(k) - self.observed_nanos) as u64,
                )
                .saturating_sub(since_receipt);
                effects.push(if left.is_zero() {
                    format!("{} 等待到期確認", k.label())
                } else {
                    format!("{} 約{:.1}秒", k.label(), left.as_secs_f64())
                });
            }
        }
        if effects.is_empty() {
            "狀態：無已確認的限時效果".into()
        } else {
            format!(
                "狀態：{}（時間估計；效果到期以伺服器確認為準）",
                effects.join("／")
            )
        }
    }
}
pub fn effective_nanos(frame: &combat::Frame, entity: u64, base: u64, pace: Pace) -> u64 {
    frame
        .timed
        .iter()
        .find(|s| s.entity_id == entity)
        .map_or(base, |s| s.effective_nanos(base, pace))
}
pub(crate) fn negotiated(response: &app::Response, enabled: bool) -> bool {
    let matches = |f: &combat::Frame| {
        f.world.feature_presence_matches(
            super::world_protocol::MatchDescriptor::TIMED,
            !f.timed.is_empty(),
            enabled,
        )
    };
    match response {
        app::Response::Shop(r) => r
            .snapshot
            .as_ref()
            .is_none_or(|s| s.timed.is_some() == enabled),
        app::Response::Combat(r) => {
            r.frame.as_ref().is_none_or(matches)
                && (r.reason != combat::Reason::StaleEffect || enabled)
        }
        app::Response::Cast(r) => {
            r.frame.as_ref().is_none_or(matches)
                && (r.reason != super::spell_protocol::Reason::StaleEffect || enabled)
        }
        app::Response::Item(r) => {
            r.frame.as_ref().is_none_or(matches)
                && (r.reason != super::item_protocol::Reason::ClassRestricted || enabled)
        }
        app::Response::Ammo(r) => r.frame.as_ref().is_none_or(matches),
        app::Response::Siege(r) => {
            r.frame.as_ref().is_none_or(matches)
                && (r.reason != super::siege_protocol::Reason::StaleEffect || enabled)
        }
        _ => true,
    }
}
pub fn validate_frame(frame: &combat::Frame) -> Result<(), ApplicationError> {
    if frame.timed.is_empty() {
        return if frame.items.iter().any(|i| {
            Kind::from_item(i.item_id).is_some()
                || i.stocks
                    .iter()
                    .any(|s| Kind::from_item(s.item_id).is_some())
        }) {
            Err(invalid())
        } else {
            Ok(())
        };
    }
    if frame.timed.len() != frame.world.actors.len()
        || frame.items.len() != frame.timed.len()
        || frame.weights.len() != frame.timed.len()
    {
        return Err(invalid());
    }
    for (index, ((s, a), items)) in frame
        .timed
        .iter()
        .zip(&frame.world.actors)
        .zip(&frame.items)
        .enumerate()
    {
        s.validate()?;
        if s.entity_id != a.entity_id || s.revision > frame.world.revision {
            return Err(invalid());
        }
        if a.identity().character().is_none() && *s != State::initial(s.entity_id) {
            return Err(invalid());
        }
        let active = s.haste() || s.brave() || s.wisdom();
        if active
            && (frame.world.phase != world::Phase::Active
                || frame.vitals[index].hp == 0
                || s.revision <= frame.vitals[index].death_revision)
            || matches!(
                frame.world.phase,
                world::Phase::Loading | world::Phase::Countdown
            ) && s.revision != 0
        {
            return Err(invalid());
        }
        if (s.haste() || s.brave())
            && (a.step_interval_nanos % 1_000_000 != 0
                || frame.vitals[index].attack_interval_nanos % 1_000_000 != 0)
        {
            return Err(invalid());
        }
        for k in Kind::ALL {
            if s.active(k)
                && (!k.eligible(frame.world.room.members[index].class)
                    || !items
                        .stock(k.item_id())
                        .is_some_and(|s| s.remaining < s.initial))
            {
                return Err(invalid());
            }
        }
        if let Some(k) = Kind::from_item(items.item_id) {
            if items.use_revision == frame.world.revision && s.revision != frame.world.revision {
                return Err(invalid());
            }
            if items.use_revision == s.revision
                && (!s.active(k)
                    || s.deadline(k) - s.observed_nanos != k.duration_millis() * 1_000_000)
            {
                return Err(invalid());
            }
        }
        if frame.world.completion.as_ref().is_some_and(|c| {
            c.event
                .engine_revision
                .checked_add(1)
                .is_none_or(|r| s.revision > r)
        }) {
            return Err(invalid());
        }
    }
    Ok(())
}
pub fn follows(a: &combat::Frame, b: &combat::Frame) -> bool {
    if a.timed.len() != b.timed.len() {
        return false;
    }
    a.timed.iter().zip(&b.timed).all(|(old, new)| {
        if old.entity_id != new.entity_id
            || new.revision < old.revision
            || new.observed_nanos < old.observed_nanos
        {
            return false;
        }
        if new.revision == old.revision {
            return old == new;
        }
        if new.revision <= a.world.revision {
            return false;
        }
        let consumed = |k: Kind| {
            a.item(new.entity_id)
                .and_then(|i| i.stock(k.item_id()))
                .zip(b.item(new.entity_id).and_then(|i| i.stock(k.item_id())))
                .is_some_and(|(a, b)| b.remaining < a.remaining)
        };
        let terminal = b.world.phase != world::Phase::Active
            || b.vital(new.entity_id).is_some_and(|v| v.hp == 0)
            || b.honor
                .as_ref()
                .and_then(|h| h.life(new.entity_id))
                .is_some_and(|l| {
                    l.last_death_revision > a.world.revision
                        && new.revision >= l.last_death_revision
                });
        let mut changed = false;
        for k in Kind::ALL {
            if new.active(k)
                && b.honor
                    .as_ref()
                    .and_then(|h| h.life(new.entity_id))
                    .is_some_and(|l| l.last_death_revision > a.world.revision)
                && (!consumed(k)
                    || new.revision <= b.vital(new.entity_id).map_or(0, |v| v.death_revision))
            {
                return false;
            }
            if new.active(k) && (!old.active(k) || new.deadline(k) > old.deadline(k)) {
                if !consumed(k) {
                    return false;
                }
            }
            if old.active(k) && (!new.active(k) || new.deadline(k) < old.deadline(k)) {
                let replaced = match k {
                    Kind::Brave => new.active(Kind::ElfBrave) && consumed(Kind::ElfBrave),
                    Kind::ElfBrave => new.active(Kind::Brave) && consumed(Kind::Brave),
                    _ => false,
                };
                if !terminal && new.observed_nanos < old.deadline(k) && !replaced {
                    return false;
                }
                // A still-active same-family deadline cannot run backwards,
                // even when another effect expires in the same observation.
                if new.active(k) && new.deadline(k) < old.deadline(k) {
                    return false;
                }
            }
            changed |= old.active(k) != new.active(k) || old.deadline(k) != new.deadline(k);
        }
        if !changed && !Kind::ALL.into_iter().any(consumed) {
            return false;
        }
        if let Some(item) = b.item(new.entity_id) {
            if let Some(k) = Kind::from_item(item.item_id) {
                if item.use_revision == new.revision
                    && new.active(k)
                    && new.deadline(k) - new.observed_nanos != k.duration_millis() * 1_000_000
                {
                    return false;
                }
            }
        }
        true
    })
}
pub(crate) fn decode_states(
    bytes: &[u8],
    actors: usize,
) -> Result<(Vec<State>, usize), ApplicationError> {
    if bytes.len() < 8 || bytes[2..8].iter().any(|b| *b != 0) {
        return Err(invalid());
    }
    let count = u16::from_be_bytes(bytes[..2].try_into().unwrap()) as usize;
    if count > 100 || count != 0 && count != actors || bytes.len() - 8 < count * 56 {
        return Err(invalid());
    }
    let mut states = Vec::with_capacity(count);
    for i in 0..count {
        let b = &bytes[8 + i * 56..8 + (i + 1) * 56];
        let read = |at| i64::from_be_bytes(b[at..at + 8].try_into().unwrap());
        if b[49..56].iter().any(|v| *v != 0) {
            return Err(invalid());
        }
        let state = State {
            entity_id: read(0) as u64,
            revision: read(8),
            observed_nanos: read(16),
            haste_until_nanos: read(24),
            brave_until_nanos: read(32),
            wisdom_until_nanos: read(40),
            brave_kind: b[48],
        };
        state.validate()?;
        if state.entity_id != i as u64 + 1 {
            return Err(invalid());
        }
        states.push(state);
    }
    Ok((states, 8 + count * 56))
}

#[cfg(test)]
#[path = "timed_protocol_tests.rs"]
pub(crate) mod tests;
