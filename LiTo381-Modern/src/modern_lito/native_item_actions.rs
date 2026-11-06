//! Original inventory action envelopes and authority-fenced requests.
//! A changed item is observable authority, not a correlated success ACK.
use super::{
    native_inventory::{Item, Owner},
    native_login::{LoginPhase, Snapshot},
};
use anyhow::{ensure, Context, Result};
use std::time::{Duration, Instant};
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Choice {
    pub owner: Owner,
    pub item: Item,
}
impl Choice {
    pub fn capture(s: &Snapshot, id: u32) -> Result<Self> {
        Ok(Self {
            owner: s.inventory.owner.clone().context("目前沒有角色背包")?,
            item: s
                .inventory
                .items
                .get(&id)
                .context("物品已不在背包")?
                .clone(),
        })
    }
    pub fn validate(&self, s: &Snapshot) -> Result<()> {
        ensure!(
            s.inventory.owner.as_ref() == Some(&self.owner),
            "角色或連線已變更，請重新選取物品"
        );
        ensure!(
            s.inventory.items.get(&self.item.id) == Some(&self.item),
            "物品已更新，請重新選取"
        );
        ensure!(self.item.id != 0 && self.item.count > 0, "物品不可用");
        Ok(())
    }
}
/// Cursor authority captured when a world-target item is armed. A portal, control
/// interruption or actor replacement invalidates it even if inventory is unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetChoice {
    choice: Choice,
    stamp: super::native_navigation::InputStamp,
}
impl TargetChoice {
    pub fn capture(s: &Snapshot, id: u32) -> Result<Self> {
        let choice = Choice::capture(s, id)?;
        target_available(s, &choice)?;
        let stamp = super::native_navigation::InputStamp::capture(choice.owner.connection, &s.world);
        Ok(Self { choice, stamp })
    }
    pub fn choice(&self) -> &Choice { &self.choice }
    pub fn validate(&self, s: &Snapshot) -> Result<()> {
        target_available(s, &self.choice)?;
        ensure!(self.stamp == super::native_navigation::InputStamp::capture(
            self.choice.owner.connection, &s.world), "場景或控制狀態已變更，請重新選取物品");
        Ok(())
    }
}
fn target_available(s: &Snapshot, choice: &Choice) -> Result<()> {
    ensure!(available(s) && s.phase.active_world()
        && !s.observer && !s.world.observer_mode, "請在世界中選取物品目標");
    choice.validate(s)?;
    ensure!(world_target_type(choice.item.use_type), "此物品不使用世界目標");
    ensure!(s.world.ready() && s.world.battle_result.is_none()
        && s.world.hp.is_some_and(|v| v[0] > 0)
        && s.world.own_actor().is_some_and(|a| !a.dead)
        && s.world.movement_blocks.is_empty(), "角色目前不能選取物品目標");
    Ok(())
}
pub fn target_request(s: &Snapshot, cursor: &TargetChoice, target: u32,
    position: [u16; 2]) -> Result<Vec<u8>> {
    cursor.validate(s)?;
    if target != 0 {
        let actor = s.world.actors.get(&target).context("物品目標已離開場景")?;
        ensure!((cursor.choice.item.use_type == 8 || !actor.dead)
            && s.world.actor_visible(target) && [actor.x, actor.y] == position,
            "物品目標已移動或失效");
    }
    if matches!(cursor.choice.item.use_type, 8 | 30) {
        ensure!(target != 0, "請選取物件目標");
        object_target_request(cursor.choice.item.id, target)
    } else {
        target_location_request(cursor.choice.item.id, target, position)
    }
}
/// Formal C02 targetLocationInventoryUse envelope: opcode, item, target, x, y.
/// Ground targeting uses target=0; there is no trailing string terminator.
pub fn target_location_request(id: u32, target: u32, position: [u16; 2]) -> Result<Vec<u8>> {
    ensure!(id != 0, "zero item object");
    let mut b = vec![0x1c];
    b.extend(id.to_le_bytes());
    b.extend(target.to_le_bytes());
    for coordinate in position { b.extend(coordinate.to_le_bytes()); }
    Ok(b)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Action {
    Use,
    Drop(u32),
    /// Formal C76 destroys the whole stack; it has no quantity field.
    Destroy,
}
pub fn default_use_type(t: u8) -> bool {
    // Verified signed-byte default branches: spellbooks, food, fruit, MPR,
    // collars and projectiles. 0xFF also includes ship tickets with another
    // envelope, so it cannot be dispatched solely from this wire byte.
    matches!(t, 0 | 1 | 2 | 11 | 18..=25 | 0xf7..=0xfa | 0xfc..=0xfe)
}
pub fn available(s: &Snapshot) -> bool {
    matches!(
        s.phase,
        LoginPhase::LobbyReady | LoginPhase::RoomReady | LoginPhase::BattlePending | LoginPhase::WorldReady
    ) && s.game_connected
        && s.inventory.owner.is_some()
}
pub fn request(s: &Snapshot, choice: &Choice, action: Action) -> Result<Vec<u8>> {
    validate_operation(s, choice)?;
    match action {
        Action::Use => match choice.item.use_type {
            3 => {
                // Original 406FF0 tests the signed charge byte before emitting cd.
                ensure!((1..=127).contains(&choice.item.charge), "此物品沒有可用次數");
                object_request(0x1c, choice.item.id)
            }
            9 => {
                ensure!(s.phase.active_world(), "請在世界中使用返回道具");
                let map = s.world.map_id.filter(|m| *m != 0).context("尚未取得目前地圖")?;
                let mut b = object_request(0x1c, choice.item.id)?;
                b.extend(map.to_le_bytes());
                b.extend([0; 4]);
                Ok(b)
            }
            10 => {
                let mut b = object_request(0x1c, choice.item.id)?;
                b.extend(super::native_skills::ClientTick::now()?.as_millis().to_le_bytes());
                Ok(b)
            }
            t if default_use_type(t) => use_request(choice.item.id),
            _ => anyhow::bail!("請選取此物品需要的目標或選項"),
        },
        Action::Destroy => object_request(0x76, choice.item.id),
        Action::Drop(count) => {
            ensure!(s.phase == LoginPhase::BattlePending, "請在世界中丟棄物品");
            ensure!(count > 0 && count <= choice.item.count, "丟棄數量超出目前持有量");
            let a = s.world.own_actor().context("own item actor")?;
            drop_request([a.x, a.y], choice.item.id, count)
        }
    }
}
fn validate_operation(s: &Snapshot, choice: &Choice) -> Result<()> {
    ensure!(available(s), "目前不能操作物品");
    ensure!(!s.observer && !s.world.observer_mode, "觀戰中不能操作物品");
    choice.validate(s)?;
    if s.phase.active_world() {
        ensure!(
            s.world.ready()
                && s.world.battle_result.is_none()
                && s.world.hp.is_some_and(|v| v[0] > 0)
                && s.world.own_actor().is_some_and(|a| !a.dead),
            "角色目前不能操作物品"
        );
        ensure!(s.world.movement_blocks.is_empty(), "目前狀態限制物品操作");
    }
    Ok(())
}

pub fn inventory_target_type(t: u8) -> bool { matches!(t, 7 | 14 | 26 | 27) }
pub fn world_target_type(t: u8) -> bool { matches!(t, 5 | 8 | 17 | 30) }
pub fn text_choice_type(t: u8) -> bool { matches!(t, 4 | 16) }

/// Captures the source before the user changes the selected inventory row.
/// World/control changes invalidate a pending option even if the item survives.
#[derive(Clone, Debug)]
pub struct Selection {
    pub source: Choice,
    phase: LoginPhase,
    stamp: super::native_navigation::InputStamp,
}
impl Selection {
    pub fn capture(s: &Snapshot, source: Choice) -> Result<Self> {
        validate_operation(s, &source)?;
        ensure!(inventory_target_type(source.item.use_type) || text_choice_type(source.item.use_type),
            "此物品不使用背包目標或文字選項");
        if source.item.use_type == 4 {
            ensure!((1..=127).contains(&source.item.charge), "此物品沒有可用次數");
        }
        Ok(Self { stamp: super::native_navigation::InputStamp::capture(source.owner.connection, &s.world),
            source, phase: s.phase })
    }
    pub fn validate(&self, s: &Snapshot) -> Result<()> {
        validate_operation(s, &self.source)?;
        ensure!(s.phase == self.phase && self.stamp == super::native_navigation::InputStamp::capture(
            self.source.owner.connection, &s.world), "物品選項已失效，請重新使用");
        Ok(())
    }
    pub fn item_request(&self, s: &Snapshot, target: &Choice) -> Result<Vec<u8>> {
        self.validate(s)?;
        ensure!(inventory_target_type(self.source.item.use_type), "此物品需要文字選項");
        target.validate(s)?;
        ensure!(target.owner == self.source.owner && target.item.id != self.source.item.id,
            "請選取另一件背包物品");
        object_target_request(self.source.item.id, target.item.id)
    }
    pub fn text_request(&self, s: &Snapshot, text: &str) -> Result<Vec<u8>> {
        self.validate(s)?;
        ensure!(text_choice_type(self.source.item.use_type), "此物品需要背包目標");
        let text = super::native_login::encode_text(text, 255)?;
        let mut b = object_request(0x1c, self.source.item.id)?;
        b.extend_from_slice(&text);
        b.push(0);
        Ok(b)
    }
}
fn object_request(opcode: u8, id: u32) -> Result<Vec<u8>> {
    ensure!(id != 0, "zero item object");
    let mut b = vec![opcode];
    b.extend(id.to_le_bytes());
    Ok(b)
}
pub fn object_target_request(id: u32, target: u32) -> Result<Vec<u8>> {
    ensure!(target != 0, "zero target object");
    let mut b = object_request(0x1c, id)?;
    b.extend(target.to_le_bytes());
    Ok(b)
}
pub fn use_request(id: u32) -> Result<Vec<u8>> {
    ensure!(id != 0, "zero item object");
    let mut b = vec![0x1c];
    b.extend(id.to_le_bytes());
    b.push(0);
    Ok(b)
}
pub fn drop_request(position: [u16; 2], id: u32, count: u32) -> Result<Vec<u8>> {
    ensure!(id != 0 && count != 0, "empty inventory drop");
    let mut b = vec![0x0c];
    for p in position {
        b.extend(p.to_le_bytes());
    }
    b.extend(id.to_le_bytes());
    b.extend(count.to_le_bytes());
    Ok(b)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Feedback {
    #[default]
    Idle,
    Pending,
    InventoryChanged,
    TargetRejected,
    SceneUpdated,
    TargetNoResponse,
    NoResponse,
    Interrupted,
}
impl Feedback {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Pending => "等待物品更新",
            Self::InventoryChanged => "已收到物品更新",
            Self::TargetRejected => "目標未接受，請重新選取",
            Self::SceneUpdated => "已收到場景更新",
            Self::TargetNoResponse => "未收到場景或目標回覆；可重新操作",
            Self::NoResponse => "未收到物品更新；可重新操作",
            Self::Interrupted => "物品操作已中止",
        }
    }
}
pub struct Pending {
    choice: Choice,
    phase: LoginPhase,
    generation: u64,
    incarnation: Option<u64>,
    revision: u64,
    since: Instant,
    targeted: bool,
    system_messages: u64,
    completed_portals: u64,
}
impl Pending {
    pub fn new(s: &Snapshot, choice: Choice, now: Instant) -> Self {
        Self {
            choice,
            phase: s.phase,
            generation: s.world.generation,
            incarnation: s.world.own_actor().map(|a| a.incarnation),
            revision: s.inventory.revision,
            since: now,
            targeted: false,
            system_messages: s.world.system_message_count,
            completed_portals: s.world.completed_portals,
        }
    }
    pub fn new_target(s: &Snapshot, choice: Choice, now: Instant) -> Self {
        Self { targeted: true, ..Self::new(s, choice, now) }
    }
    pub fn observe(&self, s: &Snapshot, now: Instant) -> Feedback {
        if s.phase != self.phase
            || !available(s)
            || s.inventory.owner.as_ref() != Some(&self.choice.owner)
        {
            return Feedback::Interrupted;
        }
        // S5A or its completed ACK lifecycle is scene authority, never an
        // inventory-consumption ACK. Unrelated world resets still interrupt.
        if self.targeted && s.world.generation != self.generation
            && (s.world.portal.is_some() || s.world.completed_portals > self.completed_portals) {
            return Feedback::SceneUpdated;
        }
        if s.phase.active_world()
            && (s.world.generation != self.generation
                || s.world.own_actor().map(|a| a.incarnation) != self.incarnation
                || s.world.battle_result.is_some()
                || s.world.own_actor().is_none_or(|a| a.dead))
        {
            return Feedback::Interrupted;
        }
        // The formal target-location branch emits these exact rejection texts.
        // An arbitrary chat/system message cannot complete an item request.
        if self.targeted && s.world.system_message_count > self.system_messages
            && s.world.last_system_message.as_ref().is_some_and(|(kind,text)| *kind == 9
                && matches!(text.as_str(), "該座標已有單位或障礙物，未執行傳送。"
                    | "該座標沒有可站立的地面，未執行傳送。")) {
            return Feedback::TargetRejected;
        }
        if s.inventory.revision > self.revision
            && s.inventory.items.get(&self.choice.item.id) != Some(&self.choice.item)
        {
            return Feedback::InventoryChanged;
        }
        if now.saturating_duration_since(self.since) >= Duration::from_secs(2) {
            if self.targeted { Feedback::TargetNoResponse } else { Feedback::NoResponse }
        } else {
            Feedback::Pending
        }
    }
}
