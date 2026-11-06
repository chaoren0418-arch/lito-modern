//! Fixed formal C14, learned book and S37/S53 effects. No local MP spending.
use super::native_world::World;
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

pub fn book(b: &[u8]) -> Result<BTreeSet<u16>> {
    ensure!(
        b.len() >= 2 && matches!(b[0], 0x1e | 0x1f),
        "skill book envelope"
    );
    let n = usize::from(b[1]);
    ensure!(n <= 28 && b.len() == n + 2, "skill book length");
    ensure!(n < 28 || b[29] & 0xf0 == 0, "skill outside 1..220");
    Ok(b[2..]
        .iter()
        .enumerate()
        .flat_map(|(row, bits)| {
            (0..8).filter_map(move |col| {
                (bits & (1 << col) != 0).then_some((row * 8 + col + 1) as u16)
            })
        })
        .collect())
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Effect {
    pub sequence: u64,
    pub graphics: u16,
    pub target: Option<(u32, u64)>,
    pub position: [u16; 2],
}
fn can_cast(world: &World, skill: u16) -> Result<()> {
    ensure!(
        world.ready() && world.battle_result.is_none() && world.hp.is_some_and(|v| v[0] > 0),
        "目前不能施法"
    );
    ensure!(world.movement_blocks.is_empty(), "目前狀態限制施法");
    ensure!(world.learned_skills.contains(&skill), "尚未學會此技能");
    ensure!(world.mp.is_some_and(|v| v[0] > 0), "MP 不足或尚未取得");
    let own = world.own_actor().context("own caster")?;
    ensure!(!own.dead, "角色已死亡");
    Ok(())
}
/// Item object IDs must never pass through the actor-target request path.
pub fn item_targeted(skill: u16) -> bool { matches!(skill, 12 | 21) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemTarget {
    choice: super::native_item_actions::Choice,
    stamp: super::native_navigation::InputStamp,
    revision: u64,
    map: Option<u16>,
}
impl ItemTarget {
    pub fn capture(s: &super::native_login::Snapshot, epoch: u64, id: u32) -> Result<Self> {
        let target = Self {
            choice: super::native_item_actions::Choice::capture(s, id)?,
            stamp: super::native_navigation::InputStamp::capture(epoch, &s.world),
            revision: s.inventory.revision,
            map: s.world.map_id,
        };
        target.validate(s, epoch)?;
        Ok(target)
    }
    pub fn validate(&self, s: &super::native_login::Snapshot, epoch: u64) -> Result<()> {
        ensure!(s.game_connected && s.phase.active_world() && !s.observer
            && !s.world.observer_mode && s.world.ready(), "目前不能選取附魔物品");
        self.choice.validate(s)?;
        ensure!(self.choice.owner.connection == epoch && self.revision == s.inventory.revision
            && self.map == s.world.map_id
            && self.stamp == super::native_navigation::InputStamp::capture(epoch, &s.world),
            "背包或場景已更新，請重新選取附魔物品");
        Ok(())
    }
}
pub fn item_request(s: &super::native_login::Snapshot, epoch: u64, skill: u16, target: &ItemTarget) -> Result<Vec<u8>> {
    ensure!(item_targeted(skill), "此技能不是物品附魔");
    target.validate(s, epoch)?;
    can_cast(&s.world, skill)?;
    // Eligibility, duration, durability, cost and inventory updates are server-owned.
    let mut bytes = vec![0x14, ((skill - 1) / 8) as u8, ((skill - 1) % 8) as u8];
    bytes.extend_from_slice(&target.choice.item.id.to_le_bytes());
    Ok(bytes)
}

/// C14/0/4 carries a destination, never an actor ID or a predicted move.
pub fn teleport_request(world: &World, destination: super::native_portal::Destination) -> Result<Vec<u8>> {
    can_cast(world, 5)?;
    ensure!(!world.observer_mode, "觀戰時不能傳送");
    let mut bytes = vec![0x14, 0, 4];
    for value in [destination.map, destination.x, destination.y] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}
pub fn ground_request(world: &World, skill: u16, position: [u16; 2]) -> Result<Vec<u8>> {
    ensure!(matches!(skill, 58 | 63), "此地面技能尚未接入");
    can_cast(world, skill)?;
    let own = world.own_actor().unwrap();
    let range = own.x.abs_diff(position[0]).max(own.y.abs_diff(position[1]));
    if skill == 58 {
        ensure!(
            (1..=8).contains(&range),
            "火牆選點須在八格內，且不可選自己所在格"
        );
    } else {
        ensure!(
            in_native_screen([own.x, own.y], position),
            "生命之泉選點超出可施放畫面"
        );
    }
    let mut b = vec![0x14, 7, if skill == 58 { 1 } else { 6 }];
    b.extend_from_slice(&position[0].to_le_bytes());
    b.extend_from_slice(&position[1].to_le_bytes());
    Ok(b)
}
/// Formal BattleSpellCatalog screen diamond, including its asymmetric edge.
pub fn in_native_screen(own: [u16; 2], target: [u16; 2]) -> bool {
    let dx = i32::from(target[0]) - i32::from(own[0]);
    let dy = i32::from(target[1]) - i32::from(own[1]);
    let distance = dx.abs() + dy.abs();
    distance <= 18 || (distance == 19 && (19..=52).contains(&((dx + 18).abs() + (dy + 18).abs())))
}
/// Packet clock value, never an object ID. The production provider is the
/// Windows unsigned uptime millisecond counter; original caller provenance is
/// separate from the verified C02 wire contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientTick(u32);
impl ClientTick {
    pub fn from_wrapping_millis(millis:u64)->Self {Self(millis as u32)}
    pub fn as_millis(self)->u32 {self.0}
    pub fn now()->Result<Self> {
        #[cfg(windows)] {
            #[link(name="kernel32")]
            extern "system" {
                #[link_name="GetTickCount"]
                fn windows_uptime_millis()->u32;
            }
            Ok(Self(unsafe {windows_uptime_millis()}))
        }
        #[cfg(not(windows))] {
            anyhow::bail!("Windows client tick provider unavailable")
        }
    }
}
/// C02 self/tick form is intentionally distinct from target-object envelopes.
pub fn self_tick_request(world:&World,skill:u16,target:Option<u32>,tick:ClientTick)->Result<Vec<u8>> {
    ensure!(skill==60,"此技能不是自行時鐘請求");
    ensure!(target.is_none(),"隱身術不接受目標");
    can_cast(world,skill)?;
    let mut bytes=vec![0x14,7,3];bytes.extend_from_slice(&tick.0.to_le_bytes());Ok(bytes)
}
pub fn request(world: &World, skill: u16, target: Option<u32>) -> Result<Vec<u8>> {
    can_cast(world, skill)?;
    let own = world.own_actor().unwrap();
    let mut b = vec![0x14, ((skill - 1) / 8) as u8, ((skill - 1) % 8) as u8];
    match skill {
        1 | 8 | 9 | 19 | 26 | 35 | 42 | 43 | 57 => {
            let id = target.unwrap_or(own.id);
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(
                !actor.dead && !super::native_obstacles::destroyed_guardian(actor)
                    && world.actor_visible(id)
                    && if matches!(skill, 8 | 9 | 26 | 42 | 43) {
                        in_native_screen([own.x, own.y], [actor.x, actor.y])
                    } else {
                        own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= 19
                    },
                "技能目標不可用"
            );
            b.extend_from_slice(&id.to_le_bytes());
        }
        14 | 31 | 32 | 49 | 52 | 54 | 72 | 78 | 206 => {ensure!(target.is_none(), "此技能不接受指定目標");}
        13 => {ensure!(target.is_none(), "偵測術不接受目標");}
        60 => {return self_tick_request(world,skill,target,ClientTick::now()?);}
        2 => {ensure!(target.is_none(), "日光術不接受目標");}
        3 => {}
        131 => {ensure!(target.is_none(),"世界樹呼喚不接受目標");}
        23 => {
            let id = target.context("請先選取怪物")?;
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(id != own.id && world.actor_visible(id) && !actor.dead
                && !super::native_obstacles::destroyed_guardian(actor)
                && in_native_screen([own.x, own.y], [actor.x, actor.y]),
                "法術目標不可用或超出畫面");
            // NPC eligibility and weakness bits are determined by the server.
            b.extend_from_slice(&id.to_le_bytes());
        }
        4 | 6 | 7 | 10 | 11 | 15 | 16 | 17 | 18 | 20 | 22 | 25 | 27 | 28 | 34 | 38 | 77 => {
            let id = target.context("請先選取法術目標")?;
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(
                id != own.id
                    && world.actor_visible(id)
                    && !actor.dead && !super::native_obstacles::destroyed_guardian(actor)
                    && own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= match skill {10=>3,11|20=>1,27|28=>4,17|22|25|34|38=>6,_=>10},
                "法術目標不可用或超出範圍"
            );
            b.extend_from_slice(&id.to_le_bytes());
            b.extend_from_slice(&actor.x.to_le_bytes());
            b.extend_from_slice(&actor.y.to_le_bytes());
        }
        _ => anyhow::bail!("此技能的請求格式尚未接入"),
    }
    Ok(b)
}
pub fn reuse_millis(skill: u16) -> u64 {
    match skill {27=>3000,22=>300,77=>5000,60=>0,11|13|17|18|20|25|49|57|72=>500,78=>12000,206=>1000,19=>100,35=>200,28=>50,_=>10}
}
pub fn client_clock_millis(skill:u16)->u64 {match skill {10|35=>200,19|28=>100,2|16|32=>10,17=>100,18|57|72=>500,206=>1200,_=>0}}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Feedback {
    #[default]
    Idle,
    Pending,
    CastAction,
    NoResponse,
    Interrupted,
    Rejected(u16),
}
impl Feedback {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Pending => "等待施法回覆",
            Self::CastAction => "收到施法動作",
            Self::NoResponse => "未收到施法回覆，可重新施放",
            Self::Interrupted => "施法已中止",
            Self::Rejected(276) => "目前地圖無法使用此技能",
            Self::Rejected(278) => "MP 不足",
            Self::Rejected(280) => "法術未生效",
            Self::Rejected(79) => "伺服器拒絕此物品附魔",
            Self::Rejected(299) => "施法材料不足",
            Self::Rejected(_) => "伺服器限制施法",
        }
    }
}
pub struct Pending {
    pub since: Instant,
    pub casts: u64,
    pub messages: u64,
    pub generation: u64,
    pub incarnation: u64,
}
impl Pending {
    pub fn observe_item(&self, world: &World, now: Instant) -> Feedback {
        let feedback = self.observe(world, now);
        if matches!(feedback, Feedback::Pending | Feedback::NoResponse)
            && world.message_count > self.messages
            && world.last_message.as_ref().is_some_and(|(code, _)| *code == 79) {
            Feedback::Rejected(79)
        } else { feedback }
    }
    pub fn observe(&self, world: &World, now: Instant) -> Feedback {
        if world.generation != self.generation
            || world.battle_result.is_some()
            || world
                .own_actor()
                .is_none_or(|a| a.incarnation != self.incarnation || a.dead)
        {
            return Feedback::Interrupted;
        }
        if let Some((sequence, code)) = world.last_skill_rejection {
            if sequence > self.messages {
                return Feedback::Rejected(code);
            }
        }
        if world.own_cast_packets > self.casts {
            return Feedback::CastAction;
        }
        if now.saturating_duration_since(self.since) >= Duration::from_secs(2) {
            return Feedback::NoResponse;
        }
        Feedback::Pending
    }
}
