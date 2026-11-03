//! Authority-confirmed equipment state. The embedded DATA remains source-persisted;
//! effective flags and traced equipment modifiers are separate, never live stats.
use super::application_protocol::{self as app, ApplicationError, SelectionContext};
use super::character_data::{self, CharacterData};
use std::sync::Arc;

pub const REQUEST_KIND: u16 = 16;
pub const RESULT_KIND: u16 = 17;
pub const MAX_FRAME_BYTES: usize = 320 * 1024;
pub const REQUEST_BYTES: usize = 88;
pub const STAT_NAMES: [&str; 39] = [
    "STR",
    "DEX",
    "CON",
    "INT",
    "WIS",
    "CHA",
    "MAX_HP",
    "MAX_MP",
    "POTION_RECOVERY",
    "SP",
    "MR",
    "MAGIC_DAMAGE",
    "MAGIC_HIT",
    "BLIND",
    "SLEEP",
    "FREEZE",
    "STONE",
    "SUSTAIN",
    "STUN",
    "STUN_SUCCESS",
    "FIRE",
    "WATER",
    "WIND",
    "EARTH",
    "REDUCTION",
    "HP_REGEN",
    "MP_REGEN",
    "WEIGHT_REDUCTION",
    "MELEE_HIT",
    "MELEE_DAMAGE",
    "RANGED_HIT",
    "RANGED_DAMAGE",
    "PIERCE",
    "AC",
    "EVASION",
    "MELEE_REDUCTION",
    "RANGED_REDUCTION",
    "IGNORE_REDUCTION",
    "PVP_REDUCTION",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Read,
    Set,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub action: Action,
    pub desired: bool,
    pub item_id: u64,
    pub expected_revision: i64,
    pub expected_view_token: [u8; 32],
}
impl Request {
    pub fn read(context: SelectionContext) -> Self {
        Self {
            context,
            action: Action::Read,
            desired: false,
            item_id: 0,
            expected_revision: 0,
            expected_view_token: [0; 32],
        }
    }
    pub fn set(snapshot: &Snapshot, item_id: u64, desired: bool) -> Self {
        Self {
            context: snapshot.source.context,
            action: Action::Set,
            desired,
            item_id,
            expected_revision: snapshot.revision,
            expected_view_token: snapshot.view_token,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.expected_revision < 0
            || match self.action {
                Action::Read => {
                    self.desired
                        || self.item_id != 0
                        || self.expected_revision != 0
                        || self.expected_view_token != [0; 32]
                }
                Action::Set => !positive_id(self.item_id),
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    Denied,
    BackendUnavailable,
    Busy,
    Conflict,
    BindingMismatch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Reason {
    InvalidRequest = 1,
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
}
impl Reason {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::InvalidRequest => "穿戴請求不合法",
            Self::UnknownItem => "此角色背包中沒有這個物品",
            Self::UnqualifiedItem => "物品資料或使用期限未通過檢查",
            Self::NotEquipment => "此物品不是可穿戴裝備",
            Self::UnqualifiedEquippedItem => "目前穿戴物品的資料未通過檢查",
            Self::InvalidLoadout => "目前裝備組合不合法",
            Self::ClassNotAllowed => "角色職業不符合裝備限制",
            Self::LevelTooLow => "角色等級低於裝備需求",
            Self::LevelTooHigh => "角色等級超過裝備上限",
            Self::Cursed => "受詛咒的裝備不能卸下",
            Self::Sealed => "封印物品不能穿戴",
            Self::SlotOccupied => "裝備欄位已被其他物品佔用",
            Self::RingLimit => "戒指數量已達上限",
            Self::AidLimit => "輔助裝備數量已達上限",
            Self::DuplicateAidCatalog => "不能同時穿戴相同種類的輔助裝備",
            Self::TalismanLimit => "護符數量已達上限",
            Self::ShieldGuarderConflict => "盾牌與臂甲不能同時穿戴",
            Self::TwoHandedShieldConflict => "雙手武器與盾牌不能同時穿戴",
            Self::LayerDependency => "請先卸下覆蓋此裝備的外層裝備",
        }
    }
    fn decode(value: u16) -> Result<Option<Self>, ApplicationError> {
        const REASONS: [Reason; 19] = [
            Reason::InvalidRequest,
            Reason::UnknownItem,
            Reason::UnqualifiedItem,
            Reason::NotEquipment,
            Reason::UnqualifiedEquippedItem,
            Reason::InvalidLoadout,
            Reason::ClassNotAllowed,
            Reason::LevelTooLow,
            Reason::LevelTooHigh,
            Reason::Cursed,
            Reason::Sealed,
            Reason::SlotOccupied,
            Reason::RingLimit,
            Reason::AidLimit,
            Reason::DuplicateAidCatalog,
            Reason::TalismanLimit,
            Reason::ShieldGuarderConflict,
            Reason::TwoHandedShieldConflict,
            Reason::LayerDependency,
        ];
        if value == 0 {
            Ok(None)
        } else {
            REASONS
                .get(value as usize - 1)
                .copied()
                .map(Some)
                .ok_or_else(invalid)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flag {
    pub item_id: u64,
    pub equipped: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaleReason {
    Missing,
    CatalogChanged,
    FactsChanged,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaleItem {
    pub item_id: u64,
    pub catalog_id: u64,
    pub reason: StaleReason,
}
/// Server-observed item lifetime, not a client-side deletion/countdown rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LifetimeState {
    Unqualified = 0,
    Permanent = 1,
    Active = 2,
    Expired = 3,
    UnstartedOrMissing = 4,
    RelativeUnqualified = 5,
    LightingUnqualified = 6,
}
impl LifetimeState {
    pub fn usable(self) -> bool {
        matches!(self, Self::Permanent | Self::Active)
    }
    fn decode(code: u8) -> Result<Self, ApplicationError> {
        match code {
            0 => Ok(Self::Unqualified),
            1 => Ok(Self::Permanent),
            2 => Ok(Self::Active),
            3 => Ok(Self::Expired),
            4 => Ok(Self::UnstartedOrMissing),
            5 => Ok(Self::RelativeUnqualified),
            6 => Ok(Self::LightingUnqualified),
            _ => Err(invalid()),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Unqualified => "期限未通過資格驗證",
            Self::Permanent => "無到期限制（伺服器確認）",
            Self::Active => "期限內（伺服器確認）",
            Self::Expired => "已到期；保留來源背包，不可裝備",
            Self::UnstartedOrMissing => "期限尚未啟動或缺少起始資料",
            Self::RelativeUnqualified => "相對期限尚未通過資格驗證",
            Self::LightingUnqualified => "照明期限尚未通過資格驗證",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeView {
    pub item_id: u64,
    pub state: LifetimeState,
    /// Conservative server-observed cutoff, NOT the raw SQL datetime. Never
    /// use the client clock to rewrite the authority's effective flags.
    pub expires_at_millis: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub revision: i64,
    pub view_token: [u8; 32],
    pub catalog_generation: [u8; 32],
    pub evaluated_at_millis: i64,
    pub next_stone_expiry: Option<i64>,
    pub modifiers: [i64; 39],
    pub magic_reduction_base: i64,
    pub source: Arc<CharacterData>,
    pub flags: Vec<Flag>,
    pub stale: Vec<StaleItem>,
    /// None only means protocol v1 did not report lifetime information.
    pub lifetimes: Option<Vec<LifetimeView>>,
    pub weight: Option<super::weight_protocol::Snapshot>,
}
impl Snapshot {
    pub fn equipped(&self, item_id: u64) -> Option<bool> {
        self.flags
            .binary_search_by_key(&item_id, |row| row.item_id)
            .ok()
            .map(|index| self.flags[index].equipped)
    }
    pub fn lifetime(&self, item_id: u64) -> Option<&LifetimeView> {
        let rows = self.lifetimes.as_ref()?;
        rows.binary_search_by_key(&item_id, |row| row.item_id)
            .ok()
            .map(|index| &rows[index])
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.source.validate()?;
        if let Some(weight) = &self.weight {
            weight.validate_stocks(std::iter::empty())?;
            let lifetimes = self.lifetimes.as_ref().ok_or_else(invalid)?;
            if lifetimes.len() != self.source.items.len() {
                return Err(invalid());
            }
            let source =
                self.source
                    .items
                    .iter()
                    .zip(lifetimes)
                    .try_fold(0i64, |sum, (item, life)| {
                        if life.item_id != item.object_id {
                            return Err(invalid());
                        }
                        if item.kind == super::character_data::ItemKind::Etc
                            || life.state == LifetimeState::Expired
                        {
                            return Ok(sum);
                        }
                        if !life.state.usable() {
                            return Err(invalid());
                        }
                        sum.checked_add(super::weight_protocol::stack_weight(
                            item.quantity,
                            item.unit_weight,
                        )?)
                        .ok_or_else(invalid)
                    })?;
            if !weight.applicable() || weight.source_gear_weight != source {
                return Err(invalid());
            }
        }
        if self.revision < 0
            || self.evaluated_at_millis < 0
            || self.next_stone_expiry.is_some_and(|value| value < 0)
            || self.flags.len() != self.source.items.len()
            || self.stale.len() > character_data::MAX_ITEMS
            || self
                .flags
                .iter()
                .zip(&self.source.items)
                .any(|(flag, item)| flag.item_id != item.object_id)
        {
            return Err(invalid());
        }
        let mut previous = 0;
        for row in &self.stale {
            if !positive_id(row.item_id) || row.item_id <= previous || !positive_id(row.catalog_id)
            {
                return Err(invalid());
            }
            previous = row.item_id;
        }
        if let Some(rows) = &self.lifetimes {
            if rows.len() != self.source.items.len()
                || self
                    .next_stone_expiry
                    .is_some_and(|next| next <= self.evaluated_at_millis)
            {
                return Err(invalid());
            }
            for ((row, item), flag) in rows.iter().zip(&self.source.items).zip(&self.flags) {
                let dated = matches!(row.state, LifetimeState::Active | LifetimeState::Expired);
                if row.item_id != item.object_id
                    || row.expires_at_millis.is_some() != dated
                    || row.expires_at_millis.is_some_and(|value| value < 0)
                    || (!row.state.usable() && flag.equipped)
                {
                    return Err(invalid());
                }
                if row.state == LifetimeState::Active {
                    let cutoff = row.expires_at_millis.ok_or_else(invalid)?;
                    if cutoff <= self.evaluated_at_millis
                        || !self.next_stone_expiry.is_some_and(|next| next <= cutoff)
                    {
                        return Err(invalid());
                    }
                }
                if row.state == LifetimeState::Expired
                    && row
                        .expires_at_millis
                        .is_some_and(|cutoff| cutoff > self.evaluated_at_millis)
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    NotRequested,
    Unsupported,
    Unavailable(Status),
    Ready(Arc<Snapshot>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub request: Request,
    pub status: Status,
    pub changed: bool,
    pub reason: Option<Reason>,
    pub blocking_item: u64,
    pub snapshot: Option<Arc<Snapshot>>,
}
impl Reply {
    /// Explain only the exact server rejection, never infer a rule from the UI.
    pub(crate) fn denial_notice(&self) -> Option<String> {
        if self.status != Status::Denied {
            return None;
        }
        let mut notice = format!("穿脫遭伺服器拒絕：{}；本次未提交。", self.reason?.label());
        if self.blocking_item != 0 {
            notice.push_str(&format!(" 阻擋物件 ID：{}。", self.blocking_item));
        }
        if let Some(row) = self
            .snapshot
            .as_ref()
            .and_then(|s| s.lifetime(self.request.item_id))
        {
            notice.push_str(&format!(
                " 物品期限：{}；來源背包仍保留，未刪除 SQL。",
                row.state.label()
            ));
        }
        Some(notice)
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.request.validate()?;
        let has_snapshot = matches!(self.status, Status::Ok | Status::Denied | Status::Conflict);
        if self.snapshot.is_some() != has_snapshot
            || self.blocking_item > i64::MAX as u64
            || (self.status == Status::Denied) != self.reason.is_some()
            || (self.status != Status::Denied && self.blocking_item != 0)
            || (self.changed && !(self.request.action == Action::Set && self.status == Status::Ok))
            || (self.request.action == Action::Read
                && matches!(self.status, Status::Denied | Status::Conflict))
        {
            return Err(invalid());
        }
        if let Some(snapshot) = &self.snapshot {
            snapshot.validate()?;
            if snapshot.source.context != self.request.context {
                return Err(invalid());
            }
            if self.status == Status::Ok && self.request.action == Action::Set {
                let revision = self
                    .request
                    .expected_revision
                    .checked_add(i64::from(self.changed))
                    .ok_or_else(invalid)?;
                if snapshot.revision != revision
                    || snapshot.equipped(self.request.item_id) != Some(self.request.desired)
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
}
fn positive_id(value: u64) -> bool {
    value > 0 && value <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid authoritative equipment frame",
    )
}

pub(crate) fn encode_request(
    request_id: u32,
    request: Request,
) -> Result<Vec<u8>, ApplicationError> {
    request.validate()?;
    let mut bytes = app::header(request_id, REQUEST_KIND, REQUEST_BYTES)?;
    bytes.extend_from_slice(&request.context.key.source_id.to_be_bytes());
    bytes.extend_from_slice(&request.context.key.object_id.to_be_bytes());
    bytes.extend_from_slice(&request.context.epoch.to_be_bytes());
    bytes.extend_from_slice(&[
        u8::from(request.action == Action::Set),
        u8::from(request.desired),
        0,
        0,
    ]);
    bytes.extend_from_slice(&request.item_id.to_be_bytes());
    bytes.extend_from_slice(&request.expected_revision.to_be_bytes());
    bytes.extend_from_slice(&request.expected_view_token);
    Ok(bytes)
}
struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.position.checked_add(count).ok_or_else(invalid)?;
        let value = self.bytes.get(self.position..end).ok_or_else(invalid)?;
        self.position = end;
        Ok(value)
    }
    fn zeros(&mut self, count: usize) -> Result<(), ApplicationError> {
        if self.take(count)?.iter().any(|byte| *byte != 0) {
            Err(invalid())
        } else {
            Ok(())
        }
    }
    fn byte(&mut self) -> Result<u8, ApplicationError> {
        Ok(self.take(1)?[0])
    }
    fn boolean(&mut self) -> Result<bool, ApplicationError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid()),
        }
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
    fn digest(&mut self) -> Result<[u8; 32], ApplicationError> {
        Ok(self.take(32)?.try_into().unwrap())
    }
}
pub(crate) fn decode_body(
    request_id: u32,
    bytes: &[u8],
    version: u16,
) -> Result<Reply, ApplicationError> {
    if bytes.len() > MAX_FRAME_BYTES - 16 || !(1..=super::protocol::MAX_VERSION).contains(&version)
    {
        return Err(invalid());
    }
    let mut input = Reader { bytes, position: 0 };
    let context = SelectionContext {
        key: app::CharacterKey {
            source_id: input.u32()?,
            object_id: input.u64()?,
        },
        epoch: input.u64()?,
    };
    let action = match input.byte()? {
        0 => Action::Read,
        1 => Action::Set,
        _ => return Err(invalid()),
    };
    let desired = input.boolean()?;
    input.zeros(2)?;
    let request = Request {
        context,
        action,
        desired,
        item_id: input.u64()?,
        expected_revision: input.i64()?,
        expected_view_token: input.digest()?,
    };
    request.validate()?;
    let status = match input.byte()? {
        0 => Status::Ok,
        1 => Status::Denied,
        2 => Status::BackendUnavailable,
        3 => Status::Busy,
        4 => Status::Conflict,
        5 => Status::BindingMismatch,
        _ => return Err(invalid()),
    };
    let changed = input.boolean()?;
    let reason = Reason::decode(input.u16()?)?;
    let blocking_item = input.u64()?;
    let present = input.boolean()?;
    input.zeros(3)?;
    let mut snapshot = if present {
        let revision = input.i64()?;
        let view_token = input.digest()?;
        let catalog_generation = input.digest()?;
        let evaluated_at_millis = input.i64()?;
        let next_stone_expiry = match input.i64()? {
            -1 => None,
            value if value >= 0 => Some(value),
            _ => return Err(invalid()),
        };
        let mut modifiers = [0; 39];
        for stat in &mut modifiers {
            *stat = input.i64()?;
        }
        let magic_reduction_base = input.i64()?;
        let flag_count = input.u16()? as usize;
        let stale_count = input.u16()? as usize;
        let length = input.u32()? as usize;
        if flag_count > character_data::MAX_ITEMS
            || stale_count > character_data::MAX_ITEMS
            || !(40..=character_data::MAX_FRAME_BYTES).contains(&length)
        {
            return Err(invalid());
        }
        let source_frame = input.take(length)?;
        // Reject nested equipment before calling the shared decoder: recursion is impossible.
        if source_frame[12..14] != character_data::RESULT_KIND.to_be_bytes() {
            return Err(invalid());
        }
        let source_response = app::decode_response(source_frame)?;
        if source_response.request_id != request_id {
            return Err(invalid());
        }
        let app::Response::CharacterData(source_reply) = source_response.response else {
            return Err(invalid());
        };
        if source_reply.context != context {
            return Err(invalid());
        }
        let source = Arc::new(source_reply.result.map_err(|_| invalid())?);
        let mut flags = Vec::with_capacity(flag_count);
        for _ in 0..flag_count {
            let item_id = input.u64()?;
            let equipped = input.boolean()?;
            input.zeros(7)?;
            flags.push(Flag { item_id, equipped });
        }
        let mut stale = Vec::with_capacity(stale_count);
        for _ in 0..stale_count {
            let item_id = input.u64()?;
            let catalog_id = input.u64()?;
            let reason = match input.byte()? {
                1 => StaleReason::Missing,
                2 => StaleReason::CatalogChanged,
                3 => StaleReason::FactsChanged,
                _ => return Err(invalid()),
            };
            input.zeros(7)?;
            stale.push(StaleItem {
                item_id,
                catalog_id,
                reason,
            });
        }
        let lifetimes = if version >= 2 {
            let count = input.u16()? as usize;
            input.zeros(2)?;
            if count != source.items.len() || count > character_data::MAX_ITEMS {
                return Err(invalid());
            }
            let mut rows = Vec::with_capacity(count);
            for _ in 0..count {
                let item_id = input.u64()?;
                let state = LifetimeState::decode(input.byte()?)?;
                input.zeros(7)?;
                let expires_at_millis = match input.i64()? {
                    -1 => None,
                    value if value >= 0 => Some(value),
                    _ => return Err(invalid()),
                };
                rows.push(LifetimeView {
                    item_id,
                    state,
                    expires_at_millis,
                });
            }
            Some(rows)
        } else {
            None
        };
        Some(Arc::new(Snapshot {
            revision,
            view_token,
            catalog_generation,
            evaluated_at_millis,
            next_stone_expiry,
            modifiers,
            magic_reduction_base,
            source,
            flags,
            stale,
            lifetimes,
            weight: None,
        }))
    } else {
        None
    };
    // The weight appendix belongs to a PRESENT equipment snapshot. Java emits
    // no appendix at all for BUSY/BACKEND_UNAVAILABLE; do not consume a phantom
    // discriminator or permit trailing data on a snapshot-less reply.
    if version >= 9 && snapshot.is_some() {
        let (weight, used) = super::weight_protocol::decode_optional(&bytes[input.position..])?;
        input.take(used)?;
        if let Some(weight) = weight {
            Arc::make_mut(snapshot.as_mut().ok_or_else(invalid)?).weight = Some(weight);
        }
    }
    if input.position != bytes.len() {
        return Err(invalid());
    }
    let result = Reply {
        request,
        status,
        changed,
        reason,
        blocking_item,
        snapshot,
    };
    result.validate()?;
    Ok(result)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    fn every_server_denial_has_a_distinct_localized_reason_and_optional_blocker() {
        let labels: std::collections::BTreeSet<_> = (1..=19)
            .map(|n| Reason::decode(n).unwrap().unwrap().label())
            .collect();
        assert_eq!(labels.len(), 19);
        let context = super::super::matchmaking_protocol::tests::context();
        let q = Request {
            action: Action::Set,
            item_id: 1,
            desired: true,
            ..Request::read(context)
        };
        let app::Response::Equipment(mut reply) =
            app::decode_response(&fixture(2, q, Status::Denied, false))
                .unwrap()
                .response
        else {
            panic!()
        };
        assert!(reply.denial_notice().unwrap().contains("職業不符合"));
        assert!(!reply.denial_notice().unwrap().contains("阻擋物件"));
        reply.reason = Some(Reason::TwoHandedShieldConflict);
        reply.blocking_item = 2;
        assert!(reply.denial_notice().unwrap().contains("雙手武器與盾牌"));
        assert!(reply.denial_notice().unwrap().contains("阻擋物件 ID：2"));
        reply.status = Status::Busy;
        assert!(reply.denial_notice().is_none());
    }
    #[test]
    fn absent_busy_and_unavailable_have_no_weight_appendix_in_any_version() {
        let context = super::super::matchmaking_protocol::tests::context();
        for status in [2u8, 3] {
            let mut packet = encode_request(1, Request::read(context)).unwrap();
            packet[4..8].copy_from_slice(&104u32.to_be_bytes());
            packet[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
            packet.extend_from_slice(&[status, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            assert_eq!(packet.len(), 104);
            for version in 1..=super::super::protocol::MAX_VERSION {
                let app::Response::Equipment(reply) =
                    app::decode_response_version(&packet, version)
                        .unwrap()
                        .response
                else {
                    panic!()
                };
                assert!(reply.snapshot.is_none());
                assert_eq!(
                    reply.status,
                    if status == 2 {
                        Status::BackendUnavailable
                    } else {
                        Status::Busy
                    }
                );
                let mut tail = packet.clone();
                tail.extend_from_slice(&[0; 4]);
                tail[4..8].copy_from_slice(&108u32.to_be_bytes());
                assert!(
                    app::decode_response_version(&tail, version).is_err(),
                    "absent appendix must not be accepted at {version}"
                );
                for n in 16..packet.len() {
                    assert!(decode_body(1, &packet[16..n], version).is_err());
                }
            }
        }
    }
    // Independent, verbatim ModernEquipmentProtocolTest / ModernCharacterDataProtocolTest golden bytes.
    const SET_HEX: &str = concat!(
        "4c544d3100000058fedcba9800100000000000070000000000000d05ffffffffffffffff01010000",
        "00000000000000630000000000000008000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    );
    const RESULT_HEX: &str = concat!(
        "4c544d3100000363fedcba9800110000000000070000000000000d05ffffffffffffffff01010000",
        "00000000000000630000000000000008000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        "000100000000000000000000010000000000000000000009202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
        "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f00000000075bcd15ffffffffffffffff",
        "7fffffffffffffff80000000000000000000000000000003fffffffffffffffc0000000000000005fffffffffffffffa",
        "0000000000000007fffffffffffffff80000000000000009fffffffffffffff6000000000000000bfffffffffffffff4",
        "000000000000000dfffffffffffffff2000000000000000ffffffffffffffff00000000000000011ffffffffffffffee",
        "0000000000000013ffffffffffffffec0000000000000015ffffffffffffffea0000000000000017ffffffffffffffe8",
        "0000000000000019ffffffffffffffe6000000000000001bffffffffffffffe4000000000000001dffffffffffffffe2",
        "000000000000001fffffffffffffffe00000000000000021ffffffffffffffde0000000000000023ffffffffffffffdc",
        "0000000000000025ffffffffffffffda000000000000002780000000000000000001000100000133",
        "4c544d3100000133fedcba98000f0000000000070000000000000d05ffffffffffffffff000000000000000000000d05",
        "00014e0003010000000000ee6b28000000000000000001000000000000000200000000000000030000000000000004ff",
        "fffffffffffffb0000000000000006000000000000000700000000000000080000000000000009000000000000000a00",
        "0000000000000b000000000000000cfffffffffffffff3000000000000000e000000000000000f000000000000001000",
        "000000000000110000000000000012000000000000001300010000000000000000006300000000000000647fffffffff",
        "ffffff8000000000000000000000000000000500000000000000ff000000000000000600000000000000070000000000",
        "00000800000000000000090005000000022431",
        "00000000000000630100000000000000000000000000009600000000000000c80200000000000000"
    );
    fn hex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
    #[test]
    fn independent_java_golden_confirms_all_39_fields_nested_data_and_stale_order() {
        let request = Request {
            context: character_data::tests::context(),
            action: Action::Set,
            desired: true,
            item_id: 99,
            expected_revision: 8,
            expected_view_token: std::array::from_fn(|i| i as u8),
        };
        assert_eq!(encode_request(0xfedcba98, request).unwrap(), hex(SET_HEX));
        let frame = hex(RESULT_HEX);
        assert_eq!(frame.len(), 867);
        let app::Response::Equipment(reply) = app::decode_response(&frame).unwrap().response else {
            panic!()
        };
        assert_eq!(reply.request, request);
        assert!(reply.changed);
        assert_eq!(reply.status, Status::Ok);
        let snapshot = reply.snapshot.unwrap();
        assert_eq!(snapshot.revision, 9);
        assert_eq!(snapshot.evaluated_at_millis, 123456789);
        assert_eq!(snapshot.next_stone_expiry, None);
        for (i, value) in snapshot.modifiers.iter().enumerate() {
            assert_eq!(
                *value,
                match i {
                    0 => i64::MAX,
                    1 => i64::MIN,
                    _ => (i as i64 + 1) * if i % 2 == 0 { 1 } else { -1 },
                }
            );
        }
        assert_eq!(snapshot.magic_reduction_base, i64::MIN);
        assert_eq!(
            snapshot.flags,
            vec![Flag {
                item_id: 99,
                equipped: true
            }]
        );
        assert_eq!(
            snapshot.stale,
            vec![StaleItem {
                item_id: 150,
                catalog_id: 200,
                reason: StaleReason::CatalogChanged
            }]
        );
        assert_eq!(snapshot.source.stats.armor_class, -5);
        assert_eq!(snapshot.source.items[0].enchantment, i64::MIN);
    }
    #[test]
    fn forged_counts_ids_nested_identity_overflow_and_unknown_codes_fail_closed() {
        let valid = hex(RESULT_HEX);
        for (offset, value) in [
            (37, 2),
            (88, 6),
            (89, 2),
            (90, 255),
            (96, 255),
            (100, 0),
            (101, 1),
            (512, 255),
            (514, 255),
            (516, 255),
            (532, 17),
            (533, 17),
            (831, 255),
            (850, 0),
            (858, 0),
            (859, 0),
            (860, 1),
        ] {
            let mut wrong = valid.clone();
            wrong[offset] = value;
            assert!(
                app::decode_response(&wrong).is_err(),
                "offset {offset} value {value}"
            );
        }
        for offset in [48, 104] {
            let mut wrong = valid.clone();
            wrong[offset..offset + 8].copy_from_slice(&i64::MAX.to_be_bytes());
            assert!(app::decode_response(&wrong).is_err());
        }
        for offset in [528, 536, 540, 548] {
            let mut wrong = valid.clone();
            wrong[offset] ^= 1;
            assert!(
                app::decode_response(&wrong).is_err(),
                "nested identity offset {offset}"
            );
        }
        let mut header = valid[..16].to_vec();
        header[4..8].copy_from_slice(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        assert!(app::read_response(&mut std::io::Cursor::new(header)).is_err());
        for length in 0..valid.len() {
            assert!(app::decode_response(&valid[..length]).is_err());
        }
    }
    pub(crate) fn fixture(
        count: usize,
        request: Request,
        status: Status,
        changed: bool,
    ) -> Vec<u8> {
        let request_id = 19;
        let mut body = encode_request(request_id, request).unwrap()[16..].to_vec();
        let status_byte = match status {
            Status::Ok => 0,
            Status::Denied => 1,
            Status::BackendUnavailable => 2,
            Status::Busy => 3,
            Status::Conflict => 4,
            Status::BindingMismatch => 5,
        };
        let present = matches!(status, Status::Ok | Status::Denied | Status::Conflict);
        body.extend_from_slice(&[
            status_byte,
            u8::from(changed),
            0,
            if status == Status::Denied { 7 } else { 0 },
        ]);
        body.extend_from_slice(&0i64.to_be_bytes());
        body.extend_from_slice(&[u8::from(present), 0, 0, 0]);
        if present {
            body.extend_from_slice(&(request.expected_revision + i64::from(changed)).to_be_bytes());
            body.extend_from_slice(&[9; 32]);
            body.extend_from_slice(&[8; 32]);
            body.extend_from_slice(&1234567i64.to_be_bytes());
            body.extend_from_slice(&(-1i64).to_be_bytes());
            for i in 0..39 {
                body.extend_from_slice(&(i64::MIN + i).to_be_bytes());
            }
            body.extend_from_slice(&i64::MAX.to_be_bytes());
            body.extend_from_slice(&(count as u16).to_be_bytes());
            body.extend_from_slice(&0u16.to_be_bytes());
            let mut data_body = character_data::tests::fixture_body_named(count, &"x".repeat(256));
            let ctx = character_data::encode_request(request_id, request.context).unwrap();
            data_body[..20].copy_from_slice(&ctx[16..]);
            data_body[24..32].copy_from_slice(&request.context.key.object_id.to_be_bytes());
            let mut data = app::header(request_id, 15, data_body.len() + 16).unwrap();
            data.extend(data_body);
            body.extend_from_slice(&(data.len() as u32).to_be_bytes());
            body.extend(data);
            for i in 1..=count {
                body.extend_from_slice(&(i as i64).to_be_bytes());
                body.extend_from_slice(&[
                    u8::from(
                        request.action == Action::Set
                            && request.item_id == i as u64
                            && request.desired,
                    ),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]);
            }
        }
        let mut frame = app::header(request_id, RESULT_KIND, body.len() + 16).unwrap();
        frame.extend(body);
        frame
    }
    pub(crate) fn sample_snapshot(context: SelectionContext) -> Arc<Snapshot> {
        let app::Response::Equipment(reply) =
            app::decode_response(&fixture(2, Request::read(context), Status::Ok, false))
                .unwrap()
                .response
        else {
            unreachable!()
        };
        reply.snapshot.unwrap()
    }
    pub(crate) fn with_lifetimes(
        mut frame: Vec<u8>,
        rows: &[(u64, LifetimeState, i64)],
    ) -> Vec<u8> {
        frame.extend_from_slice(&(rows.len() as u16).to_be_bytes());
        frame.extend_from_slice(&[0, 0]);
        for (id, state, expiry) in rows {
            frame.extend_from_slice(&id.to_be_bytes());
            frame.push(*state as u8);
            frame.extend_from_slice(&[0; 7]);
            frame.extend_from_slice(&expiry.to_be_bytes());
        }
        let length = frame.len() as u32;
        frame[4..8].copy_from_slice(&length.to_be_bytes());
        frame
    }
    #[test]
    fn version_two_lifetime_appendix_matches_java_golden_and_v1_remains_strict() {
        let mut v2 = hex(RESULT_HEX);
        v2 = with_lifetimes(v2, &[(99, LifetimeState::Permanent, -1)]);
        assert_eq!(v2.len(), 895);
        assert_eq!(
            &v2[867..],
            &hex("0001000000000000000000630100000000000000ffffffffffffffff")
        );
        assert!(app::decode_response(&v2).is_err());
        assert!(app::decode_response_version(&hex(RESULT_HEX), 2).is_err());
        let app::Response::Equipment(reply) =
            app::decode_response_version(&v2, 2).unwrap().response
        else {
            unreachable!()
        };
        let snapshot = reply.snapshot.unwrap();
        assert_eq!(
            snapshot.lifetime(99),
            Some(&LifetimeView {
                item_id: 99,
                state: LifetimeState::Permanent,
                expires_at_millis: None
            })
        );
        assert_eq!(snapshot.equipped(99), Some(true));
        let app::Response::Equipment(legacy) =
            app::decode_response(&hex(RESULT_HEX)).unwrap().response
        else {
            unreachable!()
        };
        assert!(legacy.snapshot.unwrap().lifetimes.is_none());
    }
    #[test]
    fn expired_item_stays_in_raw_inventory_and_every_state_has_exact_date_shape() {
        let request = Request::read(character_data::tests::context());
        for state in [
            LifetimeState::Unqualified,
            LifetimeState::Permanent,
            LifetimeState::Active,
            LifetimeState::Expired,
            LifetimeState::UnstartedOrMissing,
            LifetimeState::RelativeUnqualified,
            LifetimeState::LightingUnqualified,
        ] {
            let dated = matches!(state, LifetimeState::Active | LifetimeState::Expired);
            let expiry = match state {
                LifetimeState::Active => i64::MAX,
                LifetimeState::Expired => 1_234_567,
                _ => -1,
            };
            let mut base = fixture(1, request, Status::Ok, false);
            if state == LifetimeState::Active {
                base[184..192].copy_from_slice(&expiry.to_be_bytes());
            }
            let bytes = with_lifetimes(base, &[(1, state, expiry)]);
            let app::Response::Equipment(reply) =
                app::decode_response_version(&bytes, 2).unwrap().response
            else {
                unreachable!()
            };
            let snapshot = reply.snapshot.unwrap();
            assert_eq!(snapshot.source.items.len(), 1);
            assert_eq!(snapshot.source.items[0].object_id, 1);
            assert_eq!(snapshot.equipped(1), Some(false));
            assert_eq!(snapshot.lifetime(1).unwrap().state, state);
            let wrong = with_lifetimes(
                fixture(1, request, Status::Ok, false),
                &[(1, state, if dated { -1 } else { 0 })],
            );
            assert!(app::decode_response_version(&wrong, 2).is_err());
        }
        let set = Request {
            action: Action::Set,
            desired: true,
            item_id: 1,
            ..request
        };
        let wrong = with_lifetimes(
            fixture(1, set, Status::Ok, true),
            &[(1, LifetimeState::Expired, 1)],
        );
        assert!(app::decode_response_version(&wrong, 2).is_err());
    }
    #[test]
    fn lifetime_count_order_duplicates_unknown_state_padding_and_tail_are_rejected() {
        let base = fixture(
            2,
            Request::read(character_data::tests::context()),
            Status::Ok,
            false,
        );
        let valid = with_lifetimes(
            base.clone(),
            &[
                (1, LifetimeState::Permanent, -1),
                (2, LifetimeState::Expired, 5),
            ],
        );
        for rows in [
            vec![],
            vec![(1, LifetimeState::Permanent, -1)],
            vec![
                (2, LifetimeState::Permanent, -1),
                (1, LifetimeState::Expired, 5),
            ],
            vec![
                (1, LifetimeState::Permanent, -1),
                (1, LifetimeState::Expired, 5),
            ],
        ] {
            assert!(app::decode_response_version(&with_lifetimes(base.clone(), &rows), 2).is_err());
        }
        for (offset, value) in [
            (base.len() + 2, 1),
            (base.len() + 12, 7),
            (base.len() + 13, 1),
        ] {
            let mut changed = valid.clone();
            changed[offset] = value;
            assert!(app::decode_response_version(&changed, 2).is_err());
        }
        assert!(app::decode_response_version(&valid[..valid.len() - 1], 2).is_err());
        let mut extra = valid.clone();
        extra.push(0);
        let length = extra.len() as u32;
        extra[4..8].copy_from_slice(&length.to_be_bytes());
        assert!(app::decode_response_version(&extra, 2).is_err());
        assert!(app::decode_response_version(&valid, 3).is_ok());
        assert!(app::decode_response_version(&valid, 4).is_ok());
        assert!(app::decode_response_version(&valid, 5).is_ok());
        assert!(app::decode_response_version(&valid, 6).is_ok());
        assert!(app::decode_response_version(&valid, 7).is_ok());
        assert!(app::decode_response_version(&valid, 8).is_ok());
        assert!(app::decode_response_version(&valid, 9).is_err());
        assert!(app::decode_response_version(&valid, 10).is_err());
        let mut big_base = fixture(
            512,
            Request::read(character_data::tests::context()),
            Status::Ok,
            false,
        );
        big_base[184..192].copy_from_slice(&i64::MAX.to_be_bytes());
        let big = with_lifetimes(
            big_base,
            &(1..=512)
                .map(|id| (id, LifetimeState::Active, i64::MAX))
                .collect::<Vec<_>>(),
        );
        assert!(big.len() < MAX_FRAME_BYTES);
        assert!(app::decode_response_version(&big, 2).is_ok());
    }

    #[test]
    fn v9_weight_appendix_matches_source_weapons_and_armor_not_etc_or_equipped_mask() {
        use super::super::weight_protocol as weight;
        let bytes = with_lifetimes(
            fixture(
                3,
                Request::read(character_data::tests::context()),
                Status::Ok,
                false,
            ),
            &[
                (1, LifetimeState::Permanent, -1),
                (2, LifetimeState::Permanent, -1),
                (3, LifetimeState::Permanent, -1),
            ],
        );
        let app::Response::Equipment(prior) =
            app::decode_response_version(&bytes, 8).unwrap().response
        else {
            panic!()
        };
        let old = prior.snapshot.as_ref().unwrap();
        assert_eq!(old.source.items[2].kind, character_data::ItemKind::Etc);
        assert_eq!(old.equipped(1), Some(false));
        assert_eq!(old.equipped(2), Some(false));
        let source = weight::stack_weight(i64::MAX, 4)
            .unwrap()
            .checked_mul(2)
            .unwrap();
        let w = weight::Snapshot {
            source_gear_weight: source,
            supplies_weight: 0,
            total: source,
            maximum_bits: (source as f64).to_bits(),
            stage: 240,
            hit_penalty: 0,
            units: vec![],
        };
        let mut wire = bytes.clone();
        wire.extend([1, 0, 0, 0, 0, 0, 0, 0]);
        wire.extend(weight::tests::encode(&w));
        let n = wire.len() as u32;
        wire[4..8].copy_from_slice(&n.to_be_bytes());
        let app::Response::Equipment(r) = app::decode_response_version(&wire, 9).unwrap().response
        else {
            panic!()
        };
        assert_eq!(r.snapshot.as_ref().unwrap().weight, Some(w.clone()));
        assert!(weight::negotiated(
            &app::Response::Equipment(r.clone()),
            true
        ));
        assert!(!weight::negotiated(
            &app::Response::Equipment(prior.clone()),
            true
        ));
        assert!(app::decode_response_version(&wire, 8).is_err());
        let mut bad = (*r.snapshot.unwrap()).clone();
        bad.weight.as_mut().unwrap().source_gear_weight += 1;
        bad.weight.as_mut().unwrap().total += 1;
        assert!(bad.validate().is_err());
        let mut disabled = bytes;
        disabled.extend([0; 8]);
        let n = disabled.len() as u32;
        disabled[4..8].copy_from_slice(&n.to_be_bytes());
        let app::Response::Equipment(r) =
            app::decode_response_version(&disabled, 9).unwrap().response
        else {
            panic!()
        };
        assert!(r.snapshot.unwrap().weight.is_none());
        for prefix in wire.len() - 48..wire.len() {
            assert!(app::decode_response_version(&wire[..prefix], 9).is_err());
        }
    }

    #[test]
    fn confirmed_weight_excludes_expired_gear_and_zero_unit_but_rejects_unqualified_lifetimes() {
        use super::super::weight_protocol as weight;
        let bytes = with_lifetimes(
            fixture(
                3,
                Request::read(character_data::tests::context()),
                Status::Ok,
                false,
            ),
            &[
                (1, LifetimeState::Expired, 1),
                (2, LifetimeState::Permanent, -1),
                (3, LifetimeState::Unqualified, -1),
            ],
        );
        let app::Response::Equipment(r) = app::decode_response_version(&bytes, 8).unwrap().response
        else {
            panic!()
        };
        let mut s = (*r.snapshot.unwrap()).clone();
        Arc::make_mut(&mut s.source).items[1].unit_weight = 0;
        s.weight = Some(weight::Snapshot {
            source_gear_weight: 0,
            supplies_weight: 0,
            total: 0,
            maximum_bits: 1000f64.to_bits(),
            stage: 0,
            hit_penalty: 0,
            units: vec![],
        });
        s.validate().unwrap(); // raw SQL expired weapon still exists but carries no current weight.
        for state in [
            LifetimeState::Unqualified,
            LifetimeState::UnstartedOrMissing,
            LifetimeState::RelativeUnqualified,
            LifetimeState::LightingUnqualified,
        ] {
            let mut bad = s.clone();
            bad.lifetimes.as_mut().unwrap()[1].state = state;
            assert!(bad.validate().is_err());
        }
        let mut bad = s.clone();
        bad.lifetimes = None;
        assert!(bad.validate().is_err());
        let mut bad = s.clone();
        bad.lifetimes.as_mut().unwrap()[0].item_id = 2;
        assert!(bad.validate().is_err());
        let mut bad = s;
        bad.weight.as_mut().unwrap().source_gear_weight = 1;
        bad.weight.as_mut().unwrap().total = 1;
        assert!(bad.validate().is_err());
    }
    #[test]
    fn version_two_lifetime_state_and_next_deadline_match_server_evaluation_time() {
        fn timed(state: LifetimeState, cutoff: i64, evaluated: i64, next: i64) -> Vec<u8> {
            let mut base = fixture(
                1,
                Request::read(character_data::tests::context()),
                Status::Ok,
                false,
            );
            base[176..184].copy_from_slice(&evaluated.to_be_bytes());
            base[184..192].copy_from_slice(&next.to_be_bytes());
            with_lifetimes(base, &[(1, state, cutoff)])
        }
        for (state, cutoff, evaluated, next) in [
            (LifetimeState::Active, 2_000, 1_000, 1_500), // Earlier current stone is possible.
            (LifetimeState::Active, 2_000, 1_000, 2_000),
            (LifetimeState::Expired, 1_000, 1_000, -1), // Exact boundary is expired.
            (LifetimeState::Expired, 0, 0, -1),
            (LifetimeState::Permanent, -1, 1_000, 1_001), // Stone-only deadline.
            (LifetimeState::Permanent, -1, i64::MAX, -1),
            (LifetimeState::Active, i64::MAX, i64::MAX - 1, i64::MAX),
        ] {
            assert!(
                app::decode_response_version(&timed(state, cutoff, evaluated, next), 2).is_ok()
            );
        }
        for (state, cutoff, next) in [
            (LifetimeState::Active, 999, 1_500),
            (LifetimeState::Active, 1_000, 1_001),
            (LifetimeState::Active, 2_000, -1),
            (LifetimeState::Active, 2_000, 999),
            (LifetimeState::Active, 2_000, 1_000),
            (LifetimeState::Active, 2_000, 2_001),
            (LifetimeState::Expired, 1_001, -1),
            (LifetimeState::Permanent, -1, 1_000),
        ] {
            assert!(app::decode_response_version(&timed(state, cutoff, 1_000, next), 2).is_err());
        }
        // Version 1 has no lifetime-qualified deadline contract. Do not retroactively
        // impose version 2 interpretation on its legacy nextStoneExpiry value.
        let mut legacy = fixture(
            1,
            Request::read(character_data::tests::context()),
            Status::Ok,
            false,
        );
        legacy[176..184].copy_from_slice(&1_000i64.to_be_bytes());
        legacy[184..192].copy_from_slice(&1_000i64.to_be_bytes());
        assert!(app::decode_response(&legacy).is_ok());
        assert!(app::decode_response_version(
            &with_lifetimes(legacy, &[(1, LifetimeState::Permanent, -1)]),
            2
        )
        .is_err());
    }
    #[test]
    fn exact_read_request_and_signed_modifiers_match_contract() {
        let request = Request::read(character_data::tests::context());
        let bytes = encode_request(19, request).unwrap();
        assert_eq!(bytes.len(), 88);
        assert_eq!(
            &bytes[..16],
            &[76, 84, 77, 49, 0, 0, 0, 88, 0, 0, 0, 19, 0, 16, 0, 0]
        );
        assert!(bytes[36..].iter().all(|b| *b == 0));
        let snapshot = sample_snapshot(request.context);
        assert_eq!(snapshot.modifiers[33], i64::MIN + 33);
        assert_eq!(STAT_NAMES[33], "AC");
        assert_eq!(snapshot.magic_reduction_base, i64::MAX);
        assert!(snapshot.source.items[0].equipped);
        assert_eq!(snapshot.equipped(1), Some(false));
    }
    #[test]
    fn full_512_rows_exceed_old_frame_limit_without_truncation() {
        let frame = fixture(
            512,
            Request::read(character_data::tests::context()),
            Status::Ok,
            false,
        );
        assert!(frame.len() > app::MAX_FRAME_BYTES);
        assert!(frame.len() < MAX_FRAME_BYTES);
        let app::Response::Equipment(reply) = app::decode_response(&frame).unwrap().response else {
            panic!()
        };
        assert_eq!(reply.snapshot.unwrap().flags.len(), 512);
    }
    #[test]
    fn request_validation_rejects_read_extras_and_nonpositive_or_negative_set_fields() {
        let read = Request::read(character_data::tests::context());
        for wrong in [
            Request {
                desired: true,
                ..read
            },
            Request { item_id: 1, ..read },
            Request {
                expected_revision: 1,
                ..read
            },
            Request {
                expected_view_token: [1; 32],
                ..read
            },
            Request {
                action: Action::Set,
                ..read
            },
            Request {
                action: Action::Set,
                item_id: 1,
                expected_revision: -1,
                ..read
            },
            Request {
                action: Action::Set,
                item_id: u64::MAX,
                ..read
            },
        ] {
            assert!(encode_request(1, wrong).is_err());
        }
    }
    #[test]
    fn status_shape_and_revision_confirmation_are_strict() {
        let read = Request::read(character_data::tests::context());
        for status in [
            Status::Ok,
            Status::BackendUnavailable,
            Status::Busy,
            Status::BindingMismatch,
        ] {
            assert!(app::decode_response(&fixture(2, read, status, false)).is_ok());
        }
        for status in [Status::Denied, Status::Conflict] {
            assert!(app::decode_response(&fixture(2, read, status, false)).is_err());
        }
        let set = Request {
            action: Action::Set,
            item_id: 1,
            desired: true,
            expected_revision: 8,
            expected_view_token: [2; 32],
            ..read
        };
        for status in [
            Status::Ok,
            Status::Denied,
            Status::Conflict,
            Status::Busy,
            Status::BackendUnavailable,
            Status::BindingMismatch,
        ] {
            assert!(app::decode_response(&fixture(2, set, status, false)).is_ok());
        }
        assert!(app::decode_response(&fixture(2, set, Status::Ok, true)).is_ok());
        let mut wrong = fixture(2, set, Status::Ok, true);
        wrong[104..112].copy_from_slice(&8i64.to_be_bytes());
        assert!(app::decode_response(&wrong).is_err());
        let mut wrong = fixture(2, set, Status::Ok, false);
        let n = wrong.len();
        wrong[n - 24] = 0;
        assert!(app::decode_response(&wrong).is_err());
    }
    #[test]
    fn malformed_nested_frames_reserved_bits_flags_and_trailing_bytes_are_rejected() {
        let valid = fixture(
            2,
            Request::read(character_data::tests::context()),
            Status::Ok,
            false,
        );
        for offset in [
            38usize,
            103,
            103 + 8 + 32 + 32 + 8 + 8 + 39 * 8 + 8 + 2 + 2 + 4 + 13,
        ] {
            let mut wrong = valid.clone();
            wrong[offset] ^= 0xff;
            assert!(app::decode_response(&wrong).is_err(), "offset {offset}");
        }
        for length in [0, 15, 16, 87, 103, valid.len() - 1] {
            assert!(app::decode_response(&valid[..length]).is_err());
        }
        let mut wrong = valid.clone();
        let n = wrong.len();
        wrong[n - 16..n - 8].copy_from_slice(&1u64.to_be_bytes());
        assert!(app::decode_response(&wrong).is_err());
        let mut trailing = valid;
        trailing.push(0);
        let n = trailing.len() as u32;
        trailing[4..8].copy_from_slice(&n.to_be_bytes());
        assert!(app::decode_response(&trailing).is_err());
    }
}
