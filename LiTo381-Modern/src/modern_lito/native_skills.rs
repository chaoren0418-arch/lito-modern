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
    // 130 Body to Mind costs HP only (formal `useBodyToMindSpell` never reads
    // MP before the conversion), so an empty mana pool must not block it.
    ensure!(skill == 130 || hp_paid(skill) || world.mp.is_some_and(|v| v[0] > 0), "MP 不足或尚未取得");
    let own = world.own_actor().context("own caster")?;
    ensure!(!own.dead, "角色已死亡");
    Ok(())
}
/// Dragon Knight 181–188: E149 `mpCost` is 0 and the formal handlers charge
/// `hpCost` only (S10 279 when `playerHp <= hpCost`), so an empty MP bar must not
/// block the request. The HP amount itself is never predicted here.
pub fn hp_paid(skill: u16) -> bool { matches!(skill, 181..=188) }
/// Item object IDs must never pass through the actor-target request path.
pub fn item_targeted(skill: u16) -> bool { matches!(skill, 12 | 21 | 73 | 100 | 107) }

/// 12 擬似魔法武器／107 暗影之牙：正式 E149 `nativeMaterialItemWeaponEnchantBody` 讀
/// 請求的物件，檢查 `definition.weapon()` 後交 `CombatWeaponBuffs.equippedWeapon`，buff
/// 一律掛在 `inventory.equippedWeaponItem()` 上；舊的「任選背包物品」不再決定目標。
/// 客戶端因此固定送「目前裝備中的武器」的物件 ID。
pub fn weapon_bound(skill: u16) -> bool { matches!(skill, 12 | 107) }
/// 正式 `RuntimeInventory$Item.nativeViewName`：裝備中的武器在名稱後加 ` ($9)`
/// （照明 ` ($10)`、防具 ` ($117)`），之後才接次數與數量。
pub const EQUIPPED_WEAPON_MARK: &str = " ($9)";
pub fn equipped_weapon(inventory: &super::native_inventory::Inventory) -> Result<&super::native_inventory::Item> {
    let mut weapons = inventory.items.values().filter(|i| i.name.contains(EQUIPPED_WEAPON_MARK));
    let weapon = weapons.next().context("尚未裝備武器，無法施放此技能")?;
    ensure!(weapons.next().is_none(), "背包顯示多件裝備中的武器，請等待背包更新");
    Ok(weapon)
}

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
    /// 12／107 用：取目前裝備中的武器，背包或裝備一變動就失效（沿用同一份快照守衛）。
    pub fn equipped_weapon(s: &super::native_login::Snapshot, epoch: u64) -> Result<Self> {
        let id = equipped_weapon(&s.inventory)?.id;
        Self::capture(s, epoch, id)
    }
    pub fn item_id(&self) -> u32 { self.choice.item.id }
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
/// 5／69 take an explicit destination (random or a received bookmark), never
/// an actor, ground cell or a default.
pub fn destination_skill(skill: u16) -> bool { matches!(skill, 5 | 69) }
/// C14/8/4 Mass Teleport: `h(map) h(x) h(y)`, nine bytes. Random travel is the
/// current map with zero coordinates; a fixed cell must be a server bookmark.
/// MP60, one magic gem, the clan／range-3 invitations (S24/748) and the real
/// destination are server-owned. The caster's move starts only with the
/// server's S55 → C4E → S5A handshake, never from the 2236 effect.
pub fn mass_teleport_request(world: &World, destination: super::native_portal::Destination) -> Result<Vec<u8>> {
    can_cast(world, 69)?;
    ensure!(!world.observer_mode, "觀戰時不能傳送");
    let mut bytes = vec![0x14, 8, 4];
    for value in [destination.map, destination.x, destination.y] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}
/// S55 `c c(0) c(0x40) d(object)`: the server asks this client to return for a
/// reserved portal. Only this exact shape is recognised; anything else is left
/// to the existing (unhandled) path.
pub fn return_request_object(body: &[u8]) -> Option<u32> {
    (body.len() == 7 && body[..3] == [0x55, 0, 0x40])
        .then(|| u32::from_le_bytes(body[3..7].try_into().unwrap()))
        .filter(|id| *id != 0)
}
/// C4E `cc`: the formal parser reads the state byte and never trusts it.
pub const RETURN_REPLY: [u8; 2] = [0x4e, 0];
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
    matches!(skill, 14 | 31 | 32 | 49 | 52 | 53 | 54 | 59 | 62 | 70 | 72 | 78 | 80 | 88 | 89 | 90 | 91 | 97 | 98 | 99 | 101 | 102 | 104 | 105 | 106 | 109 | 110 | 111 | 112 | 114 | 115 | 117 | 129 | 130 | 134 | 137 | 138 | 146 | 147 | 150 | 154 | 155 | 156 | 159 | 161 | 162 | 164 | 169 | 171 | 172 | 175 | 176 | 181 | 182 | 185 | 186 | 206)
}
/// ④ requests whose only accepted object is the caster: formal
/// `useFireWeaponSpell`／`useStormShotSpell` answer 280 for any other ID, so the
/// client always sends its own ID and never a hovered actor.
pub fn self_bound(skill: u16) -> bool { matches!(skill, 163 | 166) }
/// Skills that take a corpse (T key cycles corpses): 41 Create Zombie, 61 Resurrection,
/// 75 Greater Resurrection and 165 Call of Nature.
pub fn corpse_targeted(skill: u16) -> bool { matches!(skill, 41 | 61 | 75 | 165) }
/// 165 Call of Nature: E149 catalog `maximumRange`, Chebyshev tiles
/// (`useCallOfNatureSpell`); no screen-diamond check on the server.
pub const CALL_OF_NATURE_RANGE: u16 = 10;
/// Casts whose map rejection (S10 353) may follow the cast action: 51 Summon
/// Monster, 154 Lesser Elemental and 162 Greater Elemental (`mapFailureMessage`).
pub fn summon_feedback(skill: u16) -> bool { matches!(skill, 51 | 154 | 162) }
/// 132 Triple Arrow is suppressed by the original client unless the caster is
/// in the bow stance; the formal server drops the request silently otherwise
/// (`walkAction != BattleAnimationCatalog.BOW_WALK`).
pub const BOW_WALK_ACTION: u8 = 20;
/// 87 Shock Stun is admitted by the original client only while the caster's
/// weapon walk family is 50 (two-hand sword) or 54 (edoryu); 91 Counter Barrier
/// only for 50. The formal handlers re-check the real equipped weapon.
pub const TWO_HAND_SWORD_WALK_ACTION: u8 = 50;
pub const EDORYU_WALK_ACTION: u8 = 54;
/// 133 Elemental Fall Down: E149 catalog `maximumRange`, checked together with
/// the native screen diamond by `useElementalFallDownSpell`.
pub const ELEMENTAL_FALL_DOWN_RANGE: u16 = 10;
/// ④ object requests whose formal handlers accept only the caster's own object ID
/// (target_to=1 with maximumRange 0: Fire Weapon, Wind Shot, Earth Skin; 168 Iron Skin
/// has target_to=0 and the same original `sub_4C9880` object form). A hovered
/// actor is never sent; the request always carries the caster.
pub fn self_object(skill: u16) -> bool { matches!(skill, 148 | 149 | 151 | 168) }
/// Requests whose tail ends in one MS950 `s` field: ⑪ True Target (113) and
/// ⑦／⑧ Call／Run Clan (116／118).
pub fn text_skill(skill: u16) -> bool { matches!(skill, 113 | 116 | 118) }
/// Formal parser limits: True Target annotation ≤0x34 bytes (may be empty);
/// clan member names 1..=0x1B bytes, already trimmed. Both reject control
/// characters and inexact MS950. Member lookup, clan and world checks stay
/// server-owned.
pub const TRUE_TARGET_TEXT_MAX: usize = 0x34;
pub const CLAN_NAME_MAX: usize = 0x1b;
/// Menu admission for 116／118 before a name is typed; `text_request` still
/// rechecks everything at send time.
pub fn admit_named(world: &World, skill: u16) -> Result<()> {
    ensure!(matches!(skill, 116 | 118), "此技能不以盟友名稱指定對象");
    can_cast(world, skill)
}
pub fn text_request(world: &World, skill: u16, target: Option<u32>, text: &str) -> Result<Vec<u8>> {
    ensure!(text_skill(skill), "此技能不帶文字欄位");
    can_cast(world, skill)?;
    let own = world.own_actor().unwrap();
    let mut b = vec![0x14, ((skill - 1) / 8) as u8, ((skill - 1) % 8) as u8];
    if skill == 113 {
        // The server re-resolves the target ID and ignores these coordinates;
        // self is an eligible candidate (BattleTrueTarget.resolve).
        let id = target.unwrap_or(own.id);
        let actor = world.actors.get(&id).context("技能目標已離開")?;
        ensure!(!actor.dead && world.actor_visible(id)
            && !super::native_obstacles::destroyed_guardian(actor)
            && in_native_screen([own.x, own.y], [actor.x, actor.y]),
            "精準目標不可用或超出畫面");
        b.extend_from_slice(&id.to_le_bytes());
        b.extend_from_slice(&actor.x.to_le_bytes());
        b.extend_from_slice(&actor.y.to_le_bytes());
        if !text.is_empty() {
            b.extend_from_slice(&super::native_login::encode_text(text, TRUE_TARGET_TEXT_MAX)
                .context("標記文字須為 MS950，最多 52 bytes，不可含控制字元")?);
        }
    } else {
        ensure!(target.is_none(), "此技能以盟友名稱指定對象");
        ensure!(!text.is_empty(), "請輸入盟友名稱");
        ensure!(text == text.trim(), "盟友名稱前後不可有空白");
        b.extend_from_slice(&super::native_login::encode_text(text, CLAN_NAME_MAX)
            .context("盟友名稱須為 MS950，最多 27 bytes，不可含控制字元")?);
    }
    b.push(0);
    Ok(b)
}

pub fn request(world: &World, skill: u16, target: Option<u32>) -> Result<Vec<u8>> {
    can_cast(world, skill)?;
    let own = world.own_actor().unwrap();
    let mut b = vec![0x14, ((skill - 1) / 8) as u8, ((skill - 1) % 8) as u8];
    // T027 batch 6 (#150) knight skills. HP costs (88: 50, 89: 120, 90／91: 100;
    // the server needs HP strictly above the cost and answers S10 279), MP,
    // durations, 90's shield／guarder check and 91's real type-3 weapon are
    // server-owned: the client never spends HP or predicts the refusal.
    if skill == 87 {
        ensure!(matches!(own.walk_action, TWO_HAND_SWORD_WALK_ACTION | EDORYU_WALK_ACTION),
            "需裝備雙手劍才能施放衝擊之暈");
    }
    if skill == 91 {
        ensure!(own.walk_action == TWO_HAND_SWORD_WALK_ACTION, "需裝備雙手劍才能施放反擊屏障");
    }
    match skill {
        // 170 Water Life (④): one living PC on the native screen, the caster included
        // (formal `useWaterLifeSpell`); PC-only eligibility and the reagent are server-owned.
        1 | 8 | 9 | 19 | 26 | 35 | 37 | 42 | 43 | 44 | 48 | 55 | 57 | 67 | 68 | 79 | 158 | 160 | 170 => {
            let id = target.unwrap_or(own.id);
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(
                !actor.dead && !super::native_obstacles::destroyed_guardian(actor)
                    && world.actor_visible(id)
                    && if skill==44 {own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y))<=3} else if matches!(skill, 8 | 9 | 26 | 37 | 42 | 43 | 44 | 48 | 55 | 67 | 68 | 79 | 158 | 160 | 170) {
                        in_native_screen([own.x, own.y], [actor.x, actor.y])
                    } else {
                        own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= 19
                    },
                "技能目標不可用"
            );
            // 79: MP20, the maximum HP／MP raise and its 1200 seconds are
            // server-owned; S37 3935 is not a status ACK.
            // 67: MP80, one gem, the control-ring `monlist` grant (sent to the
            // target's own session) or the random form are server-owned; S37 230
            // is not a polymorph ACK. Only S27 changes an appearance.
            // 55: clan／party, room／instance and action support are server-owned;
            // the client never guesses membership from names (solo self may be refused).
            // Displayed clans can reject a known mismatch, but cannot authorize
            // the server's source-character clan membership or material commit.
            if matches!(skill, 26 | 42 | 68) && id != own.id {
                ensure!(!own.clan.is_empty() && own.clan == actor.clan,
                    "此技能只能指定自己或同血盟角色");
            }
            b.extend_from_slice(&id.to_le_bytes());
        }
        id if selector_only(id) => {ensure!(target.is_none(), "此技能不接受指定目標");}
        // Without typed text: 113 sends an empty annotation, 116／118 require a name.
        id if text_skill(id) => {return text_request(world, id, target, "");}
        148 | 149 | 151 | 168 => {
            ensure!(target.is_none_or(|id| id == own.id), "此技能只能對自己施放");
            b.extend_from_slice(&own.id.to_le_bytes());
        }
        145 => {
            // Only another player's or the caster's SUMMON／TAMED companion can be
            // released; companion kind, owner, line of sight, material and the
            // probability roll are server-owned. The client only refuses targets
            // the formal handler can never resolve.
            let id = target.context("請先選取召喚物")?;
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(id != own.id && world.actor_visible(id) && !actor.dead
                && !super::native_obstacles::destroyed_guardian(actor)
                && in_native_screen([own.x, own.y], [actor.x, actor.y]),
                "釋放元素目標不可用或超出畫面");
            b.extend_from_slice(&id.to_le_bytes());
            b.extend_from_slice(&actor.x.to_le_bytes());
            b.extend_from_slice(&actor.y.to_le_bytes());
        }
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
        163 | 166 => {
            // ④ d(target) with the caster as the only formal target. Elf
            // attribute, MP and the weapon buff stay server-owned.
            ensure!(target.is_none_or(|id| id == own.id), "此技能只能對自己施放");
            b.extend_from_slice(&own.id.to_le_bytes());
        }
        157 => {
            // ⑩ d(target) h(x) h(y). The server re-resolves the target and only
            // logs stale coordinates; probability, Counter Magic, safety zones,
            // MP and the two-crystal debit stay server-owned.
            let id = target.context("請先選取大地屏障目標")?;
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(id != own.id && !actor.dead && world.actor_visible(id)
                && !super::native_obstacles::destroyed_guardian(actor)
                && in_native_screen([own.x, own.y], [actor.x, actor.y]),
                "大地屏障目標不可用或超出畫面");
            b.extend_from_slice(&id.to_le_bytes());
            b.extend_from_slice(&actor.x.to_le_bytes());
            b.extend_from_slice(&actor.y.to_le_bytes());
        }
        132 | 133 => {
            // ⑩ d(target) h(x) h(y). Bow/arrow debit, three physical rolls, MP,
            // line of sight, safety zones and the debuff roll stay server-owned.
            let id = target.context(if skill == 132 { "請先選取三重矢目標" } else { "請先選取弱化屬性目標" })?;
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(id != own.id && !actor.dead && world.actor_visible(id)
                && !super::native_obstacles::destroyed_guardian(actor)
                && in_native_screen([own.x, own.y], [actor.x, actor.y]),
                "技能目標不可用或超出畫面");
            if skill == 132 {
                ensure!(own.walk_action == BOW_WALK_ACTION, "需裝備弓才能施放三重矢");
            } else {
                ensure!(own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= ELEMENTAL_FALL_DOWN_RANGE,
                    "弱化屬性目標超出十格");
            }
            b.extend_from_slice(&id.to_le_bytes());
            b.extend_from_slice(&actor.x.to_le_bytes());
            b.extend_from_slice(&actor.y.to_le_bytes());
        }
        165 => {
            // ④ d(target): a corpse other than the caster within ten tiles.
            // Water attribute, corpse eligibility, the occupied-tile check (592),
            // MP and the crystal debit stay server-owned.
            let id = target.context("請先選取屍體")?;
            let actor = world.actors.get(&id).context("屍體已離開")?;
            ensure!(id != own.id && actor.dead && world.actor_visible(id)
                && !super::native_obstacles::destroyed_guardian(actor)
                && own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= CALL_OF_NATURE_RANGE,
                "屍體不可用或超出十格");
            b.extend_from_slice(&id.to_le_bytes());
        }
        // 61 shares 41's explicit corpse intent; the S24/321 consent, HP restore and
        // the new life are server-owned. Neither 3944 nor 230 is a success ACK.
        // 75 has the same seven-byte corpse shape; its consent is S24/322 and the
        // full-HP／experience restore is server-owned.
        41 | 61 | 75 => {
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
        // 76 Mass Slow: range 10 to the main target; the radius-3 list, two gems,
        // probability and the 64-second slow are server-owned (S37 3932 only
        // when at least one target was slowed). 87 Shock Stun: range 1; damage,
        // probability and the 2–4 second stun (S0B 4183 object) are server-owned
        // and neither the attack action nor S37 4434 is a stun ACK.
        4 | 6 | 7 | 10 | 11 | 15 | 16 | 17 | 18 | 20 | 22 | 25 | 27 | 28 | 29 | 30 | 33 | 34 | 38 | 40 | 45 | 46 | 47 | 50 | 56 | 64 | 65 | 66 | 71 | 74 | 76 | 77 | 87 | 103 | 108 | 152 | 153 | 167 | 173 | 174 | 183 | 184 | 187 | 188 => {
            // 183／184／187／188: `DragonSkillRequests.TARGETED` requires the 11-byte
            // explicit target and never guesses a nearby victim.
            let id = target.context("請先選取法術目標")?;
            let actor = world.actors.get(&id).context("技能目標已離開")?;
            ensure!(
                id != own.id
                    && world.actor_visible(id)
                    && !actor.dead && !super::native_obstacles::destroyed_guardian(actor)
                    && own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y)) <= match skill {29|30|45|167=>8,33|187=>2,40|46|47|56|64|71|173=>3,50=>6,65|66|74|76=>10,10|103|183|188=>3,11|20|87|108=>1,27|28=>4,17|22|25|34|38=>6,_=>10}
                    // 152／153／184: formal range 10 and the native screen diamond both apply.
                    // 167: original click range 8 (4CB78B, `sub_4CA6E0`); 173: formal
                    // range 3; 174: original click range 10 (4CB755) inside the diamond.
                    && (!matches!(skill, 152 | 153 | 174 | 184) || in_native_screen([own.x, own.y], [actor.x, actor.y])),
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
    match skill {97=>9990,98..=100=>0,44=>500,56|64|71=>500,59|62|66=>1500,65|70=>2000,74|76|80|88|90|91=>3000,89=>5000,87=>8000,73=>0,45=>100,46|47|50=>500,51=>1000,53=>1500,27=>3000,22=>300,33|77=>5000,60|101|104|105|106|107|108|109|110|111|113..=118|129|137|138|145|147..=152|154|155|185=>0,156|158|159|160|162|163|165|166=>0,157=>600,161=>9000,164=>300,168|172=>0,146=>1200,153=>600,167|169|170|171|176=>100,173=>1000,174=>3000,175=>300,112|182|187=>2000,130|132|133=>100,134|183|184|188=>1000,102=>2990,103=>990,68=>1500,11|13|17|18|20|25|29|30|39|40|49|57|72=>500,78=>12000,206=>1000,19=>100,35=>200,28=>50,_=>10}
}
/// 167..=176: recovered PE `word_635774[skill-1]` (600／0／100／400／300／0／1000／3000／5000／100).
pub fn client_clock_millis(skill:u16)->u64 {match skill {50|173=>1000,33|175=>5000,167=>600,169|176=>100,170=>400,171=>300,174=>3000,10|35=>200,19|28=>100,2|16|32=>10,17=>100,18|57|71|72=>500,206|188=>1200,182=>3600,183=>1800,_=>0}}
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
            Self::Rejected(79) => "對象不適用，沒有任何事情發生",
            Self::Rejected(280) => "法術未生效",
            Self::Rejected(299) => "施法材料不足",
            Self::Rejected(353) => "目前地圖無法召喚",
            Self::Rejected(743) => "等級不足以召喚此種類",
            Self::Rejected(319) => "可控制的同伴數量已達上限",
            Self::Rejected(73) => "找不到該盟友或對方不在線上",
            Self::Rejected(414) => "對象不是同血盟成員",
            Self::Rejected(629) => "對象不在同一個世界",
            Self::Rejected(647) => "目前位置無法援護盟友",
            Self::Rejected(1192) => "對方所在區域無法援護",
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
    attacks: u64,
    target: Option<super::native_skill_selection::Target>,
}
impl CastContext {
    pub(crate) fn capture(connection: u64, world: &World, skill: u16, target: Option<u32>) -> Self {
        Self { connection, generation: world.generation,
            caster: world.own_actor().map(|a| (a.id, a.incarnation)),
            corrections: world.local_corrections, interruptions: world.control_interruptions,
            skill, casts: world.own_cast_packets, attacks: world.own_attack_packets,
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
            // 187 is acknowledged by its own physical strikes, which may kill too.
            || (world.own_cast_packets == self.casts
                && !(self.skill == 187 && world.own_attack_packets > self.attacks)
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
    /// Return to Nature (145): the formal handler commits the cast action first,
    /// then sends S10 79 (not a SUMMON／TAMED companion) or 280 (roll failed). A
    /// sequenced 79 after the request is therefore a rejection even after the
    /// cast action; other casts never read it here.
    pub fn observe_release(&self, world: &World, now: Instant) -> Feedback {
        let ordinary = self.observe(world, now);
        if ordinary == Feedback::Interrupted { return ordinary; }
        if world.last_item_skill_rejection.is_some_and(|sequence| sequence > self.messages) {
            return Feedback::Rejected(79);
        }
        ordinary
    }
    /// Foe Slayer (187): the formal handler publishes no S_ACTION; after the HP
    /// commit it runs `hitCount` direct physical attacks. An own physical reply
    /// newer than `attacks` (the count captured at request time) is therefore the
    /// acknowledgement. Coded rejections and interruption keep their priority.
    pub fn observe_strikes(&self, world: &World, now: Instant, attacks: u64) -> Feedback {
        let ordinary = self.observe(world, now);
        if matches!(ordinary, Feedback::Pending | Feedback::NoResponse) && world.own_attack_packets > attacks {
            Feedback::CastAction
        } else {
            ordinary
        }
    }
    /// Call／Run Clan (116／118) also accept their sequenced name/clan/world
    /// rejections; other casts never consult `last_clan_skill_rejection`.
    pub fn observe_clan(&self, world: &World, now: Instant) -> Feedback {
        let feedback = self.observe(world, now);
        match world.last_clan_skill_rejection {
            Some((sequence, code)) if sequence > self.messages
                && matches!(feedback, Feedback::Pending | Feedback::NoResponse) => Feedback::Rejected(code),
            _ => feedback,
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
#[path = "native_t027_batch4_tests.rs"]
mod t027_batch4_tests;
#[cfg(test)]
#[path = "native_t027_batch5_tests.rs"]
mod t027_batch5_tests;
#[cfg(test)]
#[path = "native_t027_batch6_tests.rs"]
mod t027_batch6_tests;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_bound_skills_resolve_only_the_single_equipped_weapon() {
        use super::super::native_inventory::{Inventory, Item};
        let item = |id: u32, name: &str| Item { id, use_type: 1, charge: 0, icon: 1, blessing: 1, count: 1,
            identified: 1, name: name.into(), detail: vec![] };
        assert!(weapon_bound(12) && weapon_bound(107));
        assert!(!weapon_bound(21) && !weapon_bound(73) && !weapon_bound(100));
        let mut inv = Inventory::default();
        inv.items.insert(1, item(1, "$95"));
        inv.items.insert(2, item(2, "$20 ($117)"));
        inv.items.insert(3, item(3, "$100 ($10)"));
        assert!(equipped_weapon(&inv).is_err(), "armor/light marks are not weapons");
        inv.items.insert(0xfedc_ba98, item(0xfedc_ba98, "+7 $41 ($9) (3)"));
        assert_eq!(equipped_weapon(&inv).unwrap().id, 0xfedc_ba98);
        inv.items.insert(5, item(5, "$42 ($9)"));
        assert!(equipped_weapon(&inv).is_err(), "ambiguous equipment is never guessed");
    }

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
