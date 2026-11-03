//! Ground selection is tied to a rendered, qualified type9 sprite and wire actor.
//! Observed updates are not a correlated pickup ACK. Never predict ownership.
use super::{
    native_inventory::Owner,
    native_login::{LoginPhase, Snapshot},
    native_world::Actor,
    native_world_scene::Frame,
};
use anyhow::{ensure, Context, Result};
use std::time::{Duration, Instant};
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Choice {
    pub owner: Owner,
    pub generation: u64,
    pub correction: u64,
    pub caster: (u32, u64),
    pub item: Actor,
}
pub fn available(s: &Snapshot) -> bool {
    s.game_connected
        && s.phase == LoginPhase::BattlePending
        && s.inventory.owner.is_some()
        && s.world.ready()
        && s.world.battle_result.is_none()
        && s.world.hp.is_some_and(|v| v[0] > 0)
        && s.world.own_actor().is_some_and(|a| !a.dead)
        && s.world.movement_blocks.is_empty()
}
impl Choice {
    pub fn at(s: &Snapshot, frame: &Frame, x: i32, y: i32) -> Result<Self> {
        let own = s.world.own_actor().context("目前沒有世界角色")?;
        let id = frame
            .pick_current(&s.world, x, y, own.id)
            .context("此處沒有可選取的物品像素")?;
        Self::capture(s, frame, id)
    }

    pub fn capture(s: &Snapshot, frame: &Frame, id: u32) -> Result<Self> {
        ensure!(frame.input_ready(&s.world), "地面畫面已更新，請重新選取");
        let item = frame
            .ground_items
            .iter()
            .find(|a| a.id == id)
            .context("此物件不是已呈現的地面物品")?
            .clone();
        let a = s.world.own_actor().context("目前沒有世界角色")?;
        let choice = Self {
            owner: s.inventory.owner.clone().context("目前沒有角色背包")?,
            generation: s.world.generation,
            correction: s.world.local_corrections,
            caster: (a.id, a.incarnation),
            item,
        };
        choice.validate(s)?;
        Ok(choice)
    }
    pub fn validate(&self, s: &Snapshot) -> Result<()> {
        ensure!(available(s), "目前不能拾取物品");
        ensure!(
            s.inventory.owner.as_ref() == Some(&self.owner)
                && s.world.generation == self.generation
                && s.world.local_corrections == self.correction
                && s.world
                    .own_actor()
                    .is_some_and(|a| (a.id, a.incarnation) == self.caster),
            "角色或場景已變更，請重新選取"
        );
        ensure!(
            s.world.actors.get(&self.item.id) == Some(&self.item),
            "地面物品已更新，請重新選取"
        );
        ensure!(
            self.item.id != 0
                && self.item.id != self.caster.0
                && self.item.experience > 0
                && !self.item.dead
                && s.world.actor_visible(self.item.id),
            "地面物品不可用"
        );
        let own = s.world.own_actor().unwrap();
        ensure!(
            own.x.abs_diff(self.item.x).max(own.y.abs_diff(self.item.y)) <= 3,
            "請靠近地面物品（三格內）"
        );
        Ok(())
    }
}
/// Native sender uses signed subtraction followed by arithmetic shift, not u16 wrapping.
pub fn wire_x(x: u16) -> u16 {
    (32768 + (i32::from(x) - 32768).div_euclid(2)) as u16
}
pub fn request(s: &Snapshot, choice: &Choice, quantity: u32) -> Result<Vec<u8>> {
    choice.validate(s)?;
    ensure!(
        quantity > 0 && quantity <= choice.item.experience,
        "拾取數量超出地面數量"
    );
    let mut b = vec![0x0b];
    b.extend(wire_x(choice.item.x).to_le_bytes());
    b.extend(choice.item.y.to_le_bytes());
    b.extend(choice.item.id.to_le_bytes());
    b.extend(quantity.to_le_bytes());
    // The deployed battle handler does not use this field for acceptance.
    // Zero is the explicit neutral value; legacy remembered-ID membership is not inferred.
    b.push(0);
    Ok(b)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Feedback {
    #[default]
    Idle,
    Pending,
    ObservedUpdates,
    NoResponse,
    Interrupted,
}
impl Feedback {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "選取附近地面物品並輸入數量",
            Self::Pending => "等待地面及背包更新",
            Self::ObservedUpdates => "已收到地面及背包更新",
            Self::NoResponse => "未收到完整拾取更新；請確認地面及背包後重試",
            Self::Interrupted => "拾取已中止",
        }
    }
}
pub struct Pending {
    choice: Choice,
    inventory_revision: u64,
    since: Instant,
}
impl Pending {
    pub fn new(s: &Snapshot, choice: Choice, since: Instant) -> Self {
        Self {
            choice,
            inventory_revision: s.inventory.revision,
            since,
        }
    }
    pub fn observe(&self, s: &Snapshot, now: Instant) -> Feedback {
        if !available(s)
            || s.inventory.owner.as_ref() != Some(&self.choice.owner)
            || s.world.generation != self.choice.generation
            || s.world.local_corrections != self.choice.correction
            || s.world
                .own_actor()
                .is_none_or(|a| (a.id, a.incarnation) != self.choice.caster)
        {
            return Feedback::Interrupted;
        }
        if s.world.actors.get(&self.choice.item.id) != Some(&self.choice.item)
            && s.inventory.revision > self.inventory_revision
        {
            return Feedback::ObservedUpdates;
        }
        if now.saturating_duration_since(self.since) >= Duration::from_secs(2) {
            Feedback::NoResponse
        } else {
            Feedback::Pending
        }
    }
}
