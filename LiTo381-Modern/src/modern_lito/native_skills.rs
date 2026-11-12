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
        world.ready() && !world.observer_mode && world.battle_result.is_none()
            && world.hp.is_some_and(|v| v[0] > 0),
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
pub fn item_targeted(skill: u16) -> bool { matches!(skill, 12 | 21 | 73 | 100 | 107) }

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
            && !s.world.observer_mode && s.world.ready(), "目前不能選取施法物品");
        self.choice.validate(s)?;
        ensure!(self.choice.owner.connection == epoch && self.revision == s.inventory.revision
            && self.map == s.world.map_id
            && self.stamp == super::native_navigation::InputStamp::capture(epoch, &s.world),
            "背包或場景已更新，請重新選取施法物品");
        Ok(())
    }
}
pub fn item_request(s: &super::native_login::Snapshot, epoch: u64, skill: u16, target: &ItemTarget) -> Result<Vec<u8>> {
    ensure!(item_targeted(skill), "此技能不是物品目標技能");
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
/// Packet clock value, never an object ID.
///
/// Original caller (recovered PE 4271D92F): `sub_4CB5F0` switches on the
/// zero-based skill index; jump-table 4CB66E case 59 (skill 60) at 4CBAAD is
/// `call 4D69F0; push eax; push 3; push 7; push 14h; "cccd"`. So the dword is
/// the native timer sample (4D69F0), not a raw OS uptime read. The provider is
/// the shared process clock initialized at controller creation; an
/// uninitialized or failed clock rejects the request instead of substituting
/// another source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientTick(u32);
impl ClientTick {
    pub fn from_wrapping_millis(millis:u64)->Self {Self(millis as u32)}
    pub fn as_millis(self)->u32 {self.0}
    pub fn now()->Result<Self> {
        super::native_clock::sample().map(Self).context("隱身術原生時鐘不可用")
    }
}
/// C02 self/tick form is intentionally distinct from target-object envelopes.
pub fn self_tick_request(world:&World,skill:u16,target:Option<u32>,tick:ClientTick)->Result<Vec<u8>> {
    ensure!(skill==60,"此技能不是自行時鐘請求");
    ensure!(target.is_none(),"隱身術不接受目標");
    can_cast(world,skill)?;
    let mut bytes=vec![0x14,7,3];bytes.extend_from_slice(&tick.0.to_le_bytes());Ok(bytes)
}
/// One classification for wire admission and clearing unrelated hovered actors.
pub fn selector_only(skill: u16) -> bool {
    matches!(skill, 14 | 31 | 32 | 49 | 52 | 53 | 54 | 72 | 78 | 97 | 98 | 99 | 101 | 102 | 104 | 105 | 106 | 109 | 206)
}

pub fn request(world: &World, skill: u16, target: Option<u32>) -> Result<Vec<u8>> {
    can_cast(world, skill)?;
    let own = world.own_actor().unwrap();
    let mut b = vec![0x14, ((skill - 1) / 8) as u8, ((skill - 1) % 8) as u8];
    match skill {
        1 | 8 | 9 | 19 | 26 | 35 | 37 | 42 | 43 | 44 | 48 | 57 | 68 => {
            let id = target.unwrap_or(own.id);
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(
                !actor.dead && !super::native_obstacles::destroyed_guardian(actor)
                    && world.actor_visible(id)
                    && if skill==44 {own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y))<=3} else if matches!(skill, 8 | 9 | 26 | 37 | 42 | 43 | 44 | 48 | 68) {
                        in_native_screen([own.x, own.y], [actor.x, actor.y])
                    } else {
                        own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= 19
                    },
                "技能目標不可用"
            );
            // Displayed clans can reject a known mismatch, but cannot authorize
            // the server's source-character clan membership or material commit.
            if matches!(skill, 26 | 42 | 68) && id != own.id {
                ensure!(!own.clan.is_empty() && own.clan == actor.clan,
                    "此技能只能指定自己或同血盟角色");
            }
            b.extend_from_slice(&id.to_le_bytes());
        }
        id if selector_only(id) => {ensure!(target.is_none(), "此技能不接受指定目標");}
        13 => {ensure!(target.is_none(), "偵測術不接受目標");}
        60 => {return self_tick_request(world,skill,target,ClientTick::now()?);}
        2 => {ensure!(target.is_none(), "日光術不接受目標");}
        3 => {}
        131 => {ensure!(target.is_none(),"世界樹呼喚不接受目標");}
        39 => {
            // The formal mana-drain branch permits self; an invalid explicit target
            // must never silently fall back to self. HP/cost/drain stay server-owned.
            let id = target.unwrap_or(own.id);
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(!actor.dead && world.actor_visible(id)
                && !super::native_obstacles::destroyed_guardian(actor)
                && own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= 4,
                "魔力奪取目標不可用或超出四格");
            b.extend_from_slice(&id.to_le_bytes());
            b.extend_from_slice(&actor.x.to_le_bytes());
            b.extend_from_slice(&actor.y.to_le_bytes());
        }
        41 => {
            let id = target.context("請先選取屍體")?;
            let actor = world.actors.get(&id).context("屍體已離開")?;
            ensure!(id != own.id && actor.dead && world.actor_visible(id)
                && !super::native_obstacles::destroyed_guardian(actor)
                && in_native_screen([own.x, own.y], [actor.x, actor.y]),
                "屍體不可用或超出畫面");
            // Eligibility, consumption, capacity and companion creation are server-owned.
            b.extend_from_slice(&id.to_le_bytes());
        }
        23 | 36 => {
            let id = target.context("請先選取怪物")?;
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(id != own.id && world.actor_visible(id) && !actor.dead
                && !super::native_obstacles::destroyed_guardian(actor)
                && in_native_screen([own.x, own.y], [actor.x, actor.y]),
                "法術目標不可用或超出畫面");
            // NPC eligibility, taming costs and weakness bits are server-owned.
            b.extend_from_slice(&id.to_le_bytes());
            if skill == 36 {
                b.extend_from_slice(&actor.x.to_le_bytes());
                b.extend_from_slice(&actor.y.to_le_bytes());
            }
        }
        4 | 6 | 7 | 10 | 11 | 15 | 16 | 17 | 18 | 20 | 22 | 25 | 27 | 28 | 29 | 30 | 33 | 34 | 38 | 40 | 45 | 46 | 47 | 50 | 77 | 103 | 108 => {
            let id = target.context("請先選取法術目標")?;
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(
                id != own.id
                    && world.actor_visible(id)
                    && !actor.dead && !super::native_obstacles::destroyed_guardian(actor)
                    && own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= match skill {29|30|45=>8,33=>2,40|46|47=>3,50=>6,10|103=>3,11|20|108=>1,27|28=>4,17|22|25|34|38=>6,_=>10},
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
    match skill {97=>9990,98..=100=>0,44=>500,45=>100,46|47|50=>500,51=>1000,53=>1500,27=>3000,22=>300,33|77=>5000,60|101|104|105|106|107|108|109=>0,102=>2990,103=>990,68=>1500,11|13|17|18|20|25|29|30|39|40|49|57|72=>500,78=>12000,206=>1000,19=>100,35=>200,28=>50,_=>10}
}
pub fn client_clock_millis(skill:u16)->u64 {match skill {50=>1000,33=>5000,10|35=>200,19|28=>100,2|16|32=>10,17=>100,18|57|72=>500,206=>1200,_=>0}}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Feedback {
    #[default]
    Idle,
    Pending,
    CastAction,
    NoResponse,
    Interrupted,
    Rejected(u16),
    /// S_PARALYSE mode 7 ended a teleport request without S_PORTAL.
    TeleportReleased,
}
impl Feedback {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Pending => "等待施法回覆",
            Self::CastAction => "收到施法動作",
            Self::NoResponse => "未收到施法回覆，可重新施放",
            Self::Interrupted => "施法已中止",
            Self::TeleportReleased => "傳送未執行，已解除鎖定",
            Self::Rejected(276) => "目前地圖無法使用此技能",
            Self::Rejected(278) => "MP 不足",
            Self::Rejected(279) => "HP 不足",
            Self::Rejected(79) => "伺服器拒絕此物品附魔",
            Self::Rejected(280) => "法術未生效",
            Self::Rejected(299) => "施法材料不足",
            Self::Rejected(353) => "目前地圖無法召喚",
            Self::Rejected(743) => "等級不足以召喚此種類",
            Self::Rejected(319) => "可控制的同伴數量已達上限",
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
/// Captured only after a request was queued. Counters also catch a restriction
/// inserted and removed in the same transport poll; no cancellation is sent.
pub(crate) struct CastContext {
    connection: u64,
    generation: u64,
    caster: Option<(u32, u64)>,
    corrections: u64,
    interruptions: u64,
    skill: u16,
    casts: u64,
    target: Option<super::native_skill_selection::Target>,
}
impl CastContext {
    pub(crate) fn capture(connection: u64, world: &World, skill: u16, target: Option<u32>) -> Self {
        Self { connection, generation: world.generation,
            caster: world.own_actor().map(|a| (a.id, a.incarnation)),
            corrections: world.local_corrections, interruptions: world.control_interruptions,
            skill, casts: world.own_cast_packets,
            target: target.and_then(|id| super::native_skill_selection::Target::capture(world, id)) }
    }
    pub(crate) fn interrupted(&self, connection: u64, world: &World) -> bool {
        self.connection != connection || self.generation != world.generation
            || self.caster != world.own_actor().map(|a| (a.id, a.incarnation))
            || self.corrections != world.local_corrections
            || self.interruptions != world.control_interruptions
            || !world.ready() || world.battle_result.is_some()
            || !world.hp.is_some_and(|hp| hp[0] > 0)
            || world.own_actor().is_none_or(|a| a.dead)
            || !world.movement_blocks.is_empty()
            || world.movement_lock != super::native_world::MovementLock::Open
            || !world.learned_skills.contains(&self.skill)
            // An accepted cast may itself kill/remove its target in this batch.
            || (world.own_cast_packets == self.casts
                && self.target.is_some_and(|t| t.resolve(world).is_none()))
    }
}
impl Pending {
    /// Item casts (12/21/100/107) accept a sequenced S10 code 79 received after the
    /// request, even if later S10 messages in the same poll replace last_message.
    /// Ordinary casts never consult this field.
    pub fn observe_item(&self, world: &World, now: Instant) -> Feedback {
        let feedback = self.observe(world, now);
        if matches!(feedback, Feedback::Pending | Feedback::NoResponse)
            && world.last_item_skill_rejection.is_some_and(|sequence| sequence > self.messages)
        {
            Feedback::Rejected(79)
        } else {
            feedback
        }
    }
    pub fn observe(&self, world: &World, now: Instant) -> Feedback {
        if world.generation != self.generation
            || world.battle_result.is_some()
            || !world.hp.is_some_and(|hp| hp[0] > 0)
            || !world.movement_blocks.is_empty()
            || world.movement_lock != super::native_world::MovementLock::Open
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

#[derive(Clone, Copy, Debug)]
pub struct SummonSelection { pub selection: u32, pub label: &'static str, pub required_level: u16 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummonTarget {
    selection: u32,
    stamp: super::native_navigation::InputStamp,
    owner: super::native_inventory::Owner,
    revision: u64,
    map: Option<u16>,
}
impl SummonTarget {
    pub fn capture(s: &super::native_login::Snapshot, epoch: u64, selection: u32) -> Result<Self> {
        ensure!(SUMMON_SELECTIONS.iter().any(|row|row.selection==selection), "召喚選項無效");
        let value=Self { selection, stamp: super::native_navigation::InputStamp::capture(epoch,&s.world),
            owner:s.inventory.owner.clone().context("等待角色背包")?, revision:s.inventory.revision, map:s.world.map_id };
        value.validate(s,epoch)?; Ok(value)
    }
    pub fn validate(&self,s:&super::native_login::Snapshot,epoch:u64)->Result<()> {
        ensure!(s.game_connected && s.phase.active_world() && !s.observer && !s.world.observer_mode,"目前不能召喚");
        can_cast(&s.world,51)?;
        ensure!(self.owner.connection==epoch && s.inventory.owner.as_ref()==Some(&self.owner)
            && self.revision==s.inventory.revision && self.map==s.world.map_id
            && self.stamp==super::native_navigation::InputStamp::capture(epoch,&s.world),"角色、背包或場景已變更，請重新選擇召喚種類");
        Ok(())
    }
}
pub fn summon_request(s:&super::native_login::Snapshot,epoch:u64,choice:&SummonTarget)->Result<Vec<u8>> {
    choice.validate(s,epoch)?;
    let mut body=vec![0x14,6,2];body.extend(choice.selection.to_le_bytes());Ok(body)
}
impl Pending {
    /// Map/level rejection may arrive after the cast action; neither is a summon ACK.
    pub fn observe_summon(&self,world:&World,now:Instant)->Feedback {
        let ordinary=self.observe(world,now);
        if ordinary==Feedback::Interrupted {return ordinary;}
        if let Some((sequence,code))=world.last_summon_rejection {
            if sequence>self.messages {return Feedback::Rejected(code);}
        }
        ordinary
    }
    pub fn hold_summon(&self,feedback:Feedback,now:Instant)->bool {
        matches!(feedback,Feedback::Pending|Feedback::CastAction)
            && now.saturating_duration_since(self.since)<Duration::from_secs(2)
    }
}
pub const SUMMON_SELECTIONS: &[SummonSelection] = &[
    SummonSelection { selection:7, label:"哈柏哥布林", required_level:28 },
    SummonSelection { selection:263, label:"艾多倫", required_level:28 },
    SummonSelection { selection:519, label:"安普", required_level:28 },
    SummonSelection { selection:8, label:"甘地妖魔", required_level:32 },
    SummonSelection { selection:264, label:"都達瑪拉妖魔", required_level:32 },
    SummonSelection { selection:520, label:"妖魔巡守", required_level:32 },
    SummonSelection { selection:9, label:"狂野毒牙", required_level:36 },
    SummonSelection { selection:265, label:"狂野之毒", required_level:36 },
    SummonSelection { selection:521, label:"狂野之魔", required_level:36 },
    SummonSelection { selection:10, label:"食人妖精", required_level:40 },
    SummonSelection { selection:266, label:"食人妖精王", required_level:40 },
    SummonSelection { selection:522, label:"冰人", required_level:40 },
    SummonSelection { selection:11, label:"狂暴蜥蜴人", required_level:44 },
    SummonSelection { selection:267, label:"重裝蜥蜴人", required_level:44 },
    SummonSelection { selection:523, label:"高等蜥蜴人", required_level:44 },
    SummonSelection { selection:12, label:"火蜥蜴", required_level:48 },
    SummonSelection { selection:268, label:"火焰戰士", required_level:48 },
    SummonSelection { selection:524, label:"火焰弓箭手", required_level:48 },
    SummonSelection { selection:13, label:"魔熊", required_level:52 },
    SummonSelection { selection:269, label:"魔狼", required_level:52 },
    SummonSelection { selection:525, label:"魔蝙蝠", required_level:52 },
    SummonSelection { selection:14, label:"巨大守護螞蟻", required_level:56 },
    SummonSelection { selection:270, label:"強化白螞蟻群", required_level:56 },
    SummonSelection { selection:526, label:"巨大強化白螞蟻", required_level:56 },
    SummonSelection { selection:15, label:"地獄奴隸", required_level:60 },
    SummonSelection { selection:271, label:"闇精靈王", required_level:60 },
    SummonSelection { selection:527, label:"食腐獸", required_level:60 },
    SummonSelection { selection:16, label:"地獄束縛犬", required_level:64 },
    SummonSelection { selection:17, label:"變形怪", required_level:68 },
    SummonSelection { selection:18, label:"巨大牛人", required_level:72 },
    SummonSelection { selection:274, label:"黑豹", required_level:72 },
];
#[cfg(test)]
mod tests {
    use super::*;

    /// Skill 60's dword is bracketed by the shared native 4D69F0 clock, not a
    /// second OS uptime source with a different epoch or resolution.
    #[cfg(windows)]
    #[test]
    fn invisibility_tick_is_the_shared_native_clock_sample() {
        super::super::native_clock::initialize();
        for _ in 0..16 {
            let before = super::super::native_clock::sample().unwrap();
            let tick = ClientTick::now().unwrap().as_millis();
            let after = super::super::native_clock::sample().unwrap();
            assert!(tick.wrapping_sub(before) <= after.wrapping_sub(before),
                "tick {tick} outside native clock window {before}..={after}");
        }
    }

    /// Static evidence only: reads the pinned recovered PE, never executes it.
    #[test]
    #[ignore = "requires LITO_NATIVE_CLOCK_PE pointing to the pinned read-only PE"]
    fn original_invisibility_caller_sends_native_4d69f0_sample_as_cccd() {
        use sha2::{Digest, Sha256};
        let path = std::env::var_os("LITO_NATIVE_CLOCK_PE").expect("LITO_NATIVE_CLOCK_PE");
        let bytes = std::fs::read(path).expect("read original PE without executing it");
        assert_eq!(format!("{:X}", Sha256::digest(&bytes)),
            "4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9");
        let at = |va: usize, len: usize| &bytes[va - 0x400000..va - 0x400000 + len];
        let u32_at = |va: usize| u32::from_le_bytes(at(va, 4).try_into().unwrap()) as usize;
        let call = |site: usize| {
            assert_eq!(at(site, 1)[0], 0xe8, "call at {site:08X}");
            (site as u32 + 5).wrapping_add(u32_at(site + 1) as u32) as usize
        };
        // 4CB66E: movzx dl, byte_4CBD00[esi]; jmp jpt[edx*4]. esi is the
        // zero-based skill index; index 59 is skill 60 (row 7, column 3).
        assert_eq!(at(0x4cb668, 6), [0x8a, 0x96, 0x00, 0xbd, 0x4c, 0x00]);
        assert_eq!(at(0x4cb66e, 3), [0xff, 0x24, 0x95]);
        let table = u32_at(0x4cb671);
        assert_eq!(table, 0x4cbc80);
        let case = usize::from(at(0x4cbd00 + 59, 1)[0]);
        assert_eq!(u32_at(table + 4 * case), 0x4cbaad);
        // call 4D69F0; push eax; push 3; push 7; push 14h; push "cccd"; call 4C6AE0
        assert_eq!(call(0x4cbaad), 0x4d69f0);
        assert_eq!(at(0x4cbab2, 8), [0x50, 0x6a, 0x03, 0x6a, 0x07, 0x6a, 0x14, 0x68]);
        assert_eq!(at(u32_at(0x4cbaba), 5), b"cccd\0");
        assert_eq!(call(0x4cbabe), 0x4c6ae0);
        // Skills 2 (index 1) and 13 (index 12) share 4CBBD4: row/column only,
        // "ccc" with no target, tick or coordinate tail.
        for skill in [2usize, 13] {
            let case = usize::from(at(0x4cbd00 + skill - 1, 1)[0]);
            assert_eq!(u32_at(table + 4 * case), 0x4cbbd4, "skill {skill}");
        }
        assert_eq!(at(0x4cbbd4, 13),
            [0x8b, 0xce, 0x83, 0xe1, 0x07, 0x51, 0xc1, 0xfe, 0x03, 0x56, 0x6a, 0x14, 0x68]);
        assert_eq!(at(u32_at(0x4cbbe1), 4), b"ccc\0");
        assert_eq!(call(0x4cbbe5), 0x4c6ae0);
    }
}
