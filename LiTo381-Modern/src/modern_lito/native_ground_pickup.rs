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
/// 原版直接點選拾取的距離：`sub_4062E0`（0x40630B..0x406336）要求
/// `|own.x2 - item.x2| <= 2` 且 `|own.y - item.y| <= 1`。內部 X 是兩倍座標
/// （送出時 `(x - 0x8000) >> 1`），所以是遊戲座標一格（含斜角）。
/// 清單面板的三格是正式伺服器的上限，兩者分開。
pub const DIRECT_REACH: u16 = 1;
/// 原版三條拾取路徑共用的時間戳 `dword_63D5F0`：距上次送出不足 0x3DA ms 時不送、不排隊。
pub const THROTTLE: Duration = Duration::from_millis(986);
/// 走向物品的整體期限；新引擎的界線，不是原版數值。
pub const APPROACH_TIMEOUT: Duration = Duration::from_secs(20);

pub fn distance(a: [u16; 2], b: [u16; 2]) -> u16 {
    a[0].abs_diff(b[0]).max(a[1].abs_diff(b[1]))
}
/// 自己到相鄰物品的朝向；同格或不相鄰時沒有。
pub fn facing(own: [u16; 2], item: [u16; 2]) -> Option<u8> {
    const DIR: [(i32, i32); 8] = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
    let d = (i32::from(item[0]) - i32::from(own[0]), i32::from(item[1]) - i32::from(own[1]));
    DIR.iter().position(|v| *v == d).map(|h| h as u8)
}
/// `sub_4062E0` 0x4063B6..0x4063DB：物品不在腳下時，自己往物品方向的那一步
/// （`sub_48B640`，與移動同一個通行判斷）走不過去就不拾取。
pub fn unobstructed(own: [u16; 2], item: [u16; 2], can_step: impl Fn(i32, i32, u8) -> bool) -> bool {
    own == item
        || facing(own, item).is_some_and(|h| can_step(i32::from(own[0]), i32::from(own[1]), h))
}
/// 滑鼠點到地面物品之後要做的事（`sub_4062E0` 類別 9 分支）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    /// 一格內，單件或「全部拾取」：直接送整疊。
    Send(u32),
    /// 一格內，堆疊且沒有「全部拾取」：先問數量，上限是整疊。
    Prompt(u32),
    /// 一格外：先走過去，到了再判斷一次。
    Approach,
}
pub fn click(own: [u16; 2], item: [u16; 2], count: u32, pick_all: bool) -> Option<Click> {
    if count == 0 {
        return None;
    }
    if distance(own, item) > DIRECT_REACH {
        return Some(Click::Approach);
    }
    Some(if count <= 1 || pick_all { Click::Send(count) } else { Click::Prompt(count) })
}
/// 數量輸入（回呼 `sub_406620`）：空白等於整疊；0、超過整疊或不是數字都不送。
pub fn prompt_quantity(text: &str, stack: u32) -> Result<u32> {
    let text = text.trim();
    if text.is_empty() {
        ensure!(stack > 0, "地面物品已沒有數量");
        return Ok(stack);
    }
    let quantity = text.parse::<u32>().ok().context("請輸入有效的拾取數量")?;
    ensure!(quantity > 0 && quantity <= stack, "拾取數量超出地面數量");
    Ok(quantity)
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Throttle {
    last: Option<Instant>,
}
impl Throttle {
    pub fn ready(&self, now: Instant) -> bool {
        self.last.is_none_or(|at| now.saturating_duration_since(at) >= THROTTLE)
    }
    pub fn sent(&mut self, now: Instant) {
        self.last = Some(now);
    }
}
/// 附近物品清單的順序：先近後遠，同距離依 Y、X、物件 ID。
/// 原版沒有這個清單，這是新引擎的固定規則，讓同一畫面每次排出來都一樣。
pub fn order(rows: &mut [Choice], own: [u16; 2]) {
    rows.sort_by_key(|c| (distance(own, [c.item.x, c.item.y]), c.item.y, c.item.x, c.item.id));
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApproachStep {
    /// 已在一格內，可以照 `click` 的結果處理。
    Arrived,
    Step(u8),
    Ended(&'static str),
}
/// 點到一格外的地面物品後走過去（原版互動模式 3：`sub_4E2150` 走完後再呼叫
/// `sub_4062E0`）。每一步都走控制器的正常移動檢查；物品、場景或控制狀態一變就結束，
/// 不替換成別的物品，也不預測拾取結果。
#[derive(Clone, Debug)]
pub struct Approach {
    item: Actor,
    stamp: super::native_navigation::InputStamp,
    steps: u32,
    since: Instant,
}
impl Approach {
    pub fn begin(epoch: u64, world: &super::native_world::World, id: u32, now: Instant) -> Option<Self> {
        let own = world.own_actor()?;
        let item = world.actors.get(&id)?;
        (id != 0 && id != own.id && item.experience > 0 && !item.dead && world.actor_visible(id)).then(|| Self {
            item: item.clone(),
            stamp: super::native_navigation::InputStamp::capture(epoch, world),
            steps: 0,
            since: now,
        })
    }
    pub fn id(&self) -> u32 {
        self.item.id
    }
    pub fn next(
        &self,
        epoch: u64,
        world: &super::native_world::World,
        now: Instant,
        can_step: impl Fn(i32, i32, u8) -> bool,
    ) -> ApproachStep {
        let Some(own) = world.own_actor() else {
            return ApproachStep::Ended("角色或場景已變更，已停止走向物品");
        };
        if self.stamp != super::native_navigation::InputStamp::capture(epoch, world) {
            return ApproachStep::Ended("角色或場景已變更，已停止走向物品");
        }
        if world.actors.get(&self.item.id) != Some(&self.item) || !world.actor_visible(self.item.id) {
            return ApproachStep::Ended("地面物品已更新，請重新點選");
        }
        let (own, item) = ([own.x, own.y], [self.item.x, self.item.y]);
        match super::native_combat_ops::chase_step(own, item, DIRECT_REACH, can_step) {
            super::native_combat_ops::ChaseStep::InReach => ApproachStep::Arrived,
            _ if self.steps >= super::native_combat_ops::CHASE_STEP_LIMIT
                || now.saturating_duration_since(self.since) >= APPROACH_TIMEOUT =>
            {
                ApproachStep::Ended("走不到物品旁，已停止")
            }
            super::native_combat_ops::ChaseStep::Step(heading) => ApproachStep::Step(heading),
            super::native_combat_ops::ChaseStep::Blocked => ApproachStep::Ended("前方受阻，走不到物品旁"),
        }
    }
    pub fn stepped(&mut self) {
        self.steps += 1;
    }
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
            Self::Idle => "選取附近地面物品；數量留白等於整疊",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::native_world::World;

    fn packet(file: &[u8], key: &str) -> Vec<u8> {
        let v: serde_json::Value = serde_json::from_slice(file).unwrap();
        let s = v["packets"][key].as_str().unwrap();
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }
    /// 與 native_combat_ops_tests 相同的離線 fixture：自己是 1；另把 bot 改成
    /// 地面物品 77（數量 4），放在自己東邊 `gap` 格。
    fn world(gap: u16) -> World {
        let mut w = World::default();
        for key in ["world", "start", "status", "actor", "bot"] {
            w.accept(&packet(include_bytes!("../../tests/fixtures/native-world-current.json"), key)).unwrap();
        }
        let mut own = w.actors.remove(&0xfedcba98).unwrap();
        own.id = 1;
        w.status.as_mut().unwrap().object_id = 1;
        let mut item = w.actors.remove(&2).unwrap();
        item.id = 77;
        item.x = own.x + gap;
        item.y = own.y;
        item.experience = 4;
        item.dead = false;
        item.draw_state = Default::default();
        w.actors.insert(1, own);
        w.actors.insert(77, item);
        w.hp = Some([50, 50]);
        w
    }
    fn open(_: i32, _: i32, _: u8) -> bool {
        true
    }

    #[test]
    fn click_follows_the_native_reach_then_stack_order() {
        // 一格內（含斜角、腳下）：單件直接送。
        assert_eq!(click([10, 10], [11, 11], 1, false), Some(Click::Send(1)));
        assert_eq!(click([10, 10], [10, 10], 1, false), Some(Click::Send(1)));
        assert_eq!(click([10, 10], [9, 10], 1, true), Some(Click::Send(1)));
        // 堆疊：沒有「全部拾取」先問數量，有就送整疊。
        assert_eq!(click([10, 10], [10, 11], 5, false), Some(Click::Prompt(5)));
        assert_eq!(click([10, 10], [10, 11], 5, true), Some(Click::Send(5)));
        assert_eq!(click([10, 10], [10, 11], u32::MAX, true), Some(Click::Send(u32::MAX)));
        // 一格外一律先走過去，不看數量。
        assert_eq!(click([10, 10], [12, 10], 1, true), Some(Click::Approach));
        assert_eq!(click([10, 10], [11, 12], 5, false), Some(Click::Approach));
        assert_eq!(click([10, 10], [13, 13], 5, true), Some(Click::Approach));
        // 數量 0 不送也不走。
        assert_eq!(click([10, 10], [10, 10], 0, true), None);
        assert_eq!(click([10, 10], [20, 10], 0, false), None);
        assert_eq!(distance([0, 65535], [65535, 0]), 65535);
    }

    #[test]
    fn prompt_quantity_matches_the_native_callback() {
        assert_eq!(prompt_quantity("", 4).unwrap(), 4, "空白等於整疊");
        assert_eq!(prompt_quantity("  ", 4).unwrap(), 4);
        assert_eq!(prompt_quantity("3", 4).unwrap(), 3);
        assert_eq!(prompt_quantity(" 4 ", 4).unwrap(), 4);
        assert_eq!(prompt_quantity("4294967295", u32::MAX).unwrap(), u32::MAX);
        for bad in ["0", "5", "x", "-1", "1.5", "4294967296"] {
            assert!(prompt_quantity(bad, 4).is_err(), "{bad}");
        }
        assert!(prompt_quantity("", 0).is_err());
    }

    #[test]
    fn throttle_uses_the_native_986_ms_window() {
        let now = Instant::now();
        let mut throttle = Throttle::default();
        assert!(throttle.ready(now));
        throttle.sent(now);
        assert!(!throttle.ready(now));
        assert!(!throttle.ready(now + Duration::from_millis(985)));
        assert!(throttle.ready(now + Duration::from_millis(986)));
        throttle.sent(now + Duration::from_millis(986));
        assert!(!throttle.ready(now + Duration::from_millis(1971)));
        assert!(throttle.ready(now + Duration::from_millis(1972)));
    }

    #[test]
    fn facing_and_wall_check_use_the_step_toward_the_item() {
        let around = [[10, 9], [11, 9], [11, 10], [11, 11], [10, 11], [9, 11], [9, 10], [9, 9]];
        for (heading, item) in around.into_iter().enumerate() {
            assert_eq!(facing([10, 10], item), Some(heading as u8));
            assert!(unobstructed([10, 10], item, |x, y, h| (x, y) == (10, 10) && usize::from(h) == heading));
            assert!(!unobstructed([10, 10], item, |_, _, h| usize::from(h) != heading));
        }
        assert_eq!(facing([10, 10], [10, 10]), None);
        assert_eq!(facing([10, 10], [12, 10]), None);
        // 腳下不查通行；一格外不算可直接拾取。
        assert!(unobstructed([10, 10], [10, 10], |_, _, _| false));
        assert!(!unobstructed([10, 10], [12, 10], open));
    }

    #[test]
    fn nearby_rows_sort_nearest_first_then_y_x_id() {
        let w = world(1);
        let own = w.own_actor().unwrap().clone();
        let base = w.actors[&77].clone();
        let owner = Owner { connection: 2, selection: 1, character: "測試".into() };
        let row = |id: u32, dx: u16, dy: u16| {
            let mut item = base.clone();
            item.id = id;
            item.x = own.x + dx;
            item.y = own.y + dy;
            Choice { owner: owner.clone(), generation: 0, correction: 0, caster: (own.id, own.incarnation), item }
        };
        let mut rows = vec![row(9, 3, 0), row(8, 1, 1), row(7, 0, 0), row(6, 1, 0), row(5, 1, 0), row(4, 0, 2)];
        order(&mut rows, [own.x, own.y]);
        assert_eq!(rows.iter().map(|c| c.item.id).collect::<Vec<_>>(), [7, 5, 6, 8, 4, 9]);
        // 已排序的清單再排一次不變。
        let again = rows.clone();
        order(&mut rows, [own.x, own.y]);
        assert_eq!(rows, again);
    }

    #[test]
    fn approach_steps_toward_the_item_and_ends_on_any_authority_change() {
        let now = Instant::now();
        let w = world(3);
        let mut approach = Approach::begin(2, &w, 77, now).unwrap();
        assert_eq!(approach.id(), 77);
        assert_eq!(approach.next(2, &w, now, open), ApproachStep::Step(2));
        assert!(matches!(approach.next(2, &w, now, |_, _, _| false), ApproachStep::Ended(_)));
        // 走到一格內就到達；即使步數或時間已用完也一樣。
        let mut near = w.clone();
        near.actors.get_mut(&1).unwrap().x += 2;
        assert_eq!(approach.next(2, &near, now, open), ApproachStep::Arrived);
        assert_eq!(approach.next(2, &near, now + APPROACH_TIMEOUT, |_, _, _| false), ApproachStep::Arrived);
        // 期限與步數上限。
        assert!(matches!(approach.next(2, &w, now + APPROACH_TIMEOUT, open), ApproachStep::Ended(_)));
        for _ in 0..crate::modern_lito::native_combat_ops::CHASE_STEP_LIMIT {
            assert_eq!(approach.next(2, &w, now, open), ApproachStep::Step(2));
            approach.stepped();
        }
        assert!(matches!(approach.next(2, &w, now, open), ApproachStep::Ended(_)));
        // 物品數量改變、被移除、變成不可見；連線或世代改變：都結束，不換綁。
        let approach = Approach::begin(2, &w, 77, now).unwrap();
        let mut changed = w.clone();
        changed.actors.get_mut(&77).unwrap().experience = 2;
        assert!(matches!(approach.next(2, &changed, now, open), ApproachStep::Ended(_)));
        changed = w.clone();
        changed.actors.remove(&77);
        assert!(matches!(approach.next(2, &changed, now, open), ApproachStep::Ended(_)));
        changed = w.clone();
        changed.actors.get_mut(&77).unwrap().draw_state.suppressed = true;
        assert!(matches!(approach.next(2, &changed, now, open), ApproachStep::Ended(_)));
        changed = w.clone();
        changed.generation += 1;
        assert!(matches!(approach.next(2, &changed, now, open), ApproachStep::Ended(_)));
        assert!(matches!(approach.next(3, &w, now, open), ApproachStep::Ended(_)));
        // 不能對自己、0、不存在、數量 0、已失效或不可見的物件開始。
        assert!(Approach::begin(2, &w, 1, now).is_none());
        assert!(Approach::begin(2, &w, 0, now).is_none());
        assert!(Approach::begin(2, &w, 78, now).is_none());
        for spoil in [0usize, 1, 2] {
            let mut bad = w.clone();
            let item = bad.actors.get_mut(&77).unwrap();
            match spoil {
                0 => item.experience = 0,
                1 => item.dead = true,
                _ => item.draw_state.suppressed = true,
            }
            assert!(Approach::begin(2, &bad, 77, now).is_none(), "{spoil}");
        }
    }
}