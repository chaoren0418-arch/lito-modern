//! Read-only private UI from the exact observation retained by painted pixels.
//! Never merges saved HP/food with live vitals, or treats this view as SET/USE.
use super::{
    battle_follow::Observation,
    battle_inventory::{Key, Selection},
    battle_window,
    world_window::DetailsPage,
};
use std::fmt::Write;

#[derive(Clone, Copy)]
enum InventoryView {
    All,
    Selected(Option<Selection>),
}
impl InventoryView {
    fn includes(self, key: Key) -> bool {
        match self {
            Self::All => true,
            Self::Selected(selected) => selected.is_some_and(|s| s.key == key),
        }
    }
}
pub(crate) fn text(
    o: &Observation,
    page: DetailsPage,
    label: impl Fn(&str) -> String,
) -> Option<String> {
    build_text(o, page, label, InventoryView::All)
}

/// Full-VIP authority can be inspected without granting shared-scene LOAD or
/// ordinary battle commands. The 31 runtime fields are not source saved stats.
pub(crate) fn vip_character(o: &Observation) -> Option<String> {
    let data = o.data();
    let vip = data.vip.as_ref()?;
    if vip.owner.player != data.personal.player || vip.owner.entity != data.personal.entity {
        return None;
    }
    let actor = data.actors.0.iter().find(|a| a.entity == vip.owner.entity)?;
    let member = data.scene_member(actor).ok()?;
    let mut out = String::new();
    let _ = write!(out,
        "{}({}) · 場次 {}／世代 {} · {:?}\r\n",
        member.name.0, member.source_name.0, data.world.identity.match_id,
        data.world.identity.generation, data.world.phase);
    out.push_str("完整 VIP 本場權威數值（唯讀）。共享戰鬥畫面與操作尚未開放；此頁不代表可進場，也不提供穿脫。\r\n\r\n");
    let _ = write!(out, "即時狀態：{}\r\n", battle_window::health(o));
    let _ = write!(out,
        "來源 {}／角色 {}／實體 {}／生命 {}\r\n保存版本 {}／能力版本 {}／生命週期版本 {}\r\n",
        vip.owner.player.source, vip.owner.player.object, vip.owner.entity, vip.owner.life,
        vip.owner.versions.saved, vip.owner.versions.runtime,
        vip.owner.versions.lifecycle);
    out.push_str("\r\n【本場執行中 31 項能力；非來源保存值】\r\n");
    const LABELS: [&str; 31] = [
        "力量", "敏捷", "體質", "智力", "精神", "魅力", "防禦等級", "最大 HP", "最大 MP",
        "HP 回復", "MP 回復", "魔法防禦", "法術強度", "近戰傷害", "近戰命中", "遠程傷害",
        "遠程命中", "傷害減免", "魔法減免", "火", "風", "地", "水", "暈眩", "石化",
        "睡眠", "冰凍", "持續", "致盲", "經驗倍率", "金幣倍率",
    ];
    for ((name, label), value) in super::battle_payload::vip::FIELD_NAMES
        .iter().zip(LABELS).zip(vip.owner.abilities.0.iter()) {
        let _ = write!(out, "{label}（{name}）：{value}\r\n");
    }
    let _ = write!(out,
        "\r\n已確認 VIP 物品 {} 件／狀態圖示 {} 個；圖示僅在共享資源資格化後於上方顯示，裝備操作仍未開放。\r\n",
        vip.owner.items.0.len(), vip.owner.icons.0.len());
    (!out.contains('\0') && out.encode_utf16().count() <= super::world_window::MAX_DETAILS_UTF16)
        .then_some(out)
}

/// The full-VIP owner and the source inventory are one validated READ, not a
/// cached lobby inventory. This page deliberately contains no command proof.
pub(crate) fn vip_inventory(o: &Observation, label: impl Fn(&str) -> String) -> Option<String> {
    let data = o.data();
    let vip = data.vip.as_ref()?;
    if vip.owner.player != data.personal.player || vip.owner.entity != data.personal.entity {
        return None;
    }
    let source = data.personal.equipment.as_ref()?;
    let mut out = String::new();
    let _ = write!(out,
        "完整 VIP 本場來源背包（唯讀） · 來源 {}／角色 {} · 場次 {}／世代 {} · {:?}\r\n保存版本 {}／能力版本 {}／生命週期版本 {}\r\n",
        vip.owner.player.source, vip.owner.player.object,
        data.world.identity.match_id, data.world.identity.generation, data.world.phase,
        vip.owner.versions.saved, vip.owner.versions.runtime, vip.owner.versions.lifecycle);
    out.push_str("此頁只顯示同一份已確認 READ 的來源物品與已套用 VIP 模板；不提供穿脫、使用、交易或共享場景 LOAD。狀態圖示 ID 不是可直接載入的圖號。\r\n\r\n");
    let _ = write!(out, "來源物品 {} 筆；其中已套用 VIP {} 件。\r\n",
        source.inventory.0.len(), vip.owner.items.0.len());
    if source.inventory.0.is_empty() {
        out.push_str("本場確認來源背包為空。\r\n");
    }
    for item in &source.inventory.0 {
        let active = vip.owner.items.0.iter().find(|v| v.object == item.object);
        let equipped = match o.personal().equipped(item.object) {
            Some(true) => "已裝備（本場確認）",
            Some(false) => "背包（本場確認）",
            None => "裝備狀態未提供",
        };
        let name: String = label(&item.name.0)
            .chars().filter(|c| !c.is_control()).take(240).collect();
        let _ = write!(out,
            "\r\n[{}] {} · {:?}\r\n實例 {}／目錄 {}／數量 {}／強化 {}／背包圖 {}\r\n",
            equipped, name, item.kind, item.object, item.catalog, item.quantity,
            item.enchantment, item.inventory_graphic);
        if let Some(lifetime) = o.personal().lifetime(item.object) {
            let _ = write!(out, "期限 {:?}／伺服器截止值 {}（Unix 毫秒）\r\n",
                lifetime.state, lifetime.expires_at_millis);
        }
        if let Some(active) = active {
            let template = &active.applied;
            let _ = write!(out,
                "已套用 VIP：模板 {}／互斥類 {}／狀態 effect ID {}／變身圖號 {}\r\n",
                template.catalog, template.exclusive_type, template.icon, template.polymorph);
            for (field, value) in super::battle_payload::vip::FIELD_NAMES
                .iter().zip(template.abilities.0.iter()) {
                if *value != 0 {
                    let _ = write!(out, "  {field}：{value}\r\n");
                }
            }
        }
    }
    (!out.contains('\0') && out.encode_utf16().count() <= super::world_window::MAX_DETAILS_UTF16)
        .then_some(out)
}

/// Exact learned-skill list from the same validated full-VIP READ as its
/// owner. It grants no cast proof, target selection or shared-scene LOAD.
pub(crate) fn vip_skills(o: &Observation, label: impl Fn(&str) -> String) -> Option<String> {
    use super::battle_payload::data::Support;
    let data = o.data();
    let vip = data.vip.as_ref()?;
    if vip.owner.player != data.personal.player || vip.owner.entity != data.personal.entity {
        return None;
    }
    let mut out = String::new();
    let _ = write!(out,
        "完整 VIP 本場已學技能（唯讀） · 來源 {}／角色 {} · 場次 {}／世代 {} · {:?}\r\n保存版本 {}／能力版本 {}／生命週期版本 {}\r\n",
        vip.owner.player.source, vip.owner.player.object,
        data.world.identity.match_id, data.world.identity.generation, data.world.phase,
        vip.owner.versions.saved, vip.owner.versions.runtime, vip.owner.versions.lifecycle);
    out.push_str("本頁只呈現同一份已確認 READ 的技能；不是可施放許可。完整 VIP 共享場景及其操作仍未開放，按此頁或選中文字不會送出施法、扣魔或指定目標。\r\n\r\n");
    let skills = o.personal().skills();
    let _ = write!(out, "本場確認已學 {} 項。\r\n", skills.len());
    if skills.is_empty() { out.push_str("本場確認技能清單為空；不改用來源保存清單。\r\n"); }
    for skill in skills {
        let availability = match skill.support {
            Support::Teleport => "已資格化瞬間移動來源；此頁不可施放",
            Support::LightArrow => "已資格化光箭來源；此頁不可施放",
            Support::BasicHeal => "已資格化初級治癒來源；此頁不可施放",
            Support::FireStorm => "已資格化火風暴來源；此頁不可施放",
            Support::Disintegrate => "已資格化究極光裂術來源；此頁不可施放",
            Support::Unimplemented => "已學但本場尚未實作／不可施放",
            Support::UnsupportedTemplate => "來源模板未資格化／不可施放",
        };
        let name: String = label(&skill.name.0).chars()
            .filter(|c| !c.is_control()).take(240).collect();
        let _ = write!(out,
            "#{} {} · {} · MP {} · 來源重用間隔 {}ms · 原始名稱 token：{}\r\n",
            skill.id, name, availability, skill.mp_cost,
            skill.reuse_millis, skill.name.0);
    }
    (!out.contains('\0') && out.encode_utf16().count() <= super::world_window::MAX_DETAILS_UTF16)
        .then_some(out)
}

/// Exact equipped-instance focus for the separate character ledger. The
/// source has no native equipment-slot identity, so never assign one here.
pub(crate) fn character(
    o: &Observation,
    selected: Option<u64>,
    label: impl Fn(&str) -> String,
) -> Option<String> {
    let mut out = text(o, DetailsPage::Character, &label)?;
    out.push_str("\r\n【已裝備物品：圖示不代表原生裝備槽位】\r\n");
    if let Some(source) = o.personal().source() {
        let equipped: Vec<_> = source.items.iter().filter(|item| {
            i64::try_from(item.object_id).ok().and_then(|id| o.personal().equipped(id)) == Some(true)
                && matches!(item.kind, crate::modern_lito::character_data::ItemKind::Weapon
                    | crate::modern_lito::character_data::ItemKind::Armor)
        }).collect();
        let _ = write!(out, "伺服器確認 {} 件；槽位對應未提供，點下方獨立圖示清單只查看，不穿脫。\r\n", equipped.len());
        if let Some(item) = selected.and_then(|id| equipped.iter().find(|item| item.object_id == id)) {
            let name: String = label(&item.name_token).chars().filter(|c| !c.is_control()).take(240).collect();
            let _ = write!(out, "所選來源實例 {} · {} · {:?} · 背包圖 {} · 強化 {}\r\n",
                item.object_id, name, item.kind, item.inventory_graphic, item.enchantment);
        }
    } else {
        out.push_str("來源裝備資料尚未提供；不推定為空。\r\n");
    }
    (!out.contains('\0') && out.encode_utf16().count() <= super::world_window::MAX_DETAILS_UTF16)
        .then_some(out)
}

/// Only the canonical inventory controls use focused details. Legacy read-only
/// pages retain their complete list; this function never authorizes a command.
pub(crate) fn inventory(
    o: &Observation,
    selected: Option<Selection>,
    label: impl Fn(&str) -> String,
) -> Option<String> {
    if let Some(s) = selected {
        if !s.current_owner(battle_window::Stamp::of(o)) {
            return None;
        }
        let present = match s.key {
            Key::Source(id) => o
                .personal()
                .source()
                .is_some_and(|source| source.items.iter().any(|i| i.object_id == id)),
            Key::Supply(id) => o
                .personal()
                .supplies()
                .is_some_and(|s| s.stocks.0.iter().any(|s| s.item == id)),
            Key::Ammo(id) => o
                .personal()
                .ammunition()
                .is_some_and(|a| a.stocks.iter().any(|s| s.facts.item_id == id)),
            Key::Material(id) => o
                .data()
                .personal
                .materials
                .as_ref()
                .is_some_and(|m| m.stocks.0.iter().any(|s| s.item == id)),
        };
        if !present {
            return None;
        }
    }
    build_text(
        o,
        DetailsPage::Inventory,
        label,
        InventoryView::Selected(selected),
    )
}

fn build_text(
    o: &Observation,
    page: DetailsPage,
    label: impl Fn(&str) -> String,
    inventory: InventoryView,
) -> Option<String> {
    let d = o.data();
    let a = d.actors.0.iter().find(|a| a.entity == d.personal.entity)?;
    let member = d.scene_member(a).ok()?;
    let mut text = format!(
        "{}({}) · 場次 {}／世代 {}／世界版本 {} · {:?}\r\nEsc 關閉檢視；返回房間仍需伺服器確認。\r\n\r\n",
        member.name.0, member.source_name.0, d.world.identity.match_id,
        d.world.identity.generation, d.world.revision, d.world.phase);
    match page {
        DetailsPage::Character => {
            text.push_str("【本場即時能力：與目前畫面同一份確認資料】\r\n");
            text.push_str(&battle_window::health(o));
            text.push_str("\r\n");
            text.push_str(&battle_window::weight(o));
            text.push_str("\r\n");
            text.push_str(&battle_window::buffs(o));
            if let Some(v) = &a.vital {
                let _ = write!(
                    text,
                    "\r\n武器類型 {}／攻擊間隔 {} ns\r\n",
                    v.weapon_type, v.attack_interval_nanos
                );
            }
            text.push_str("\r\n【來源保存資料：不是即時戰鬥能力】\r\n");
            if let Some(e) = &d.personal.equipment {
                let s = &e.saved;
                let _ = write!(text, "保存 HP {}/{}／MP {}/{}／AC {}\r\n力量 {}／敏捷 {}／體質 {}／智力 {}／精神 {}／魅力 {}\r\n經驗 {}／正義 {}\r\n保存飽食度 {}/225（不是 HP、MP 或即時飽食度）\r\n",
                    s.saved_hp, s.maximum_hp, s.saved_mp, s.maximum_mp, s.ac,
                    s.strength, s.dexterity, s.constitution, s.intelligence, s.wisdom, s.charisma,
                    s.experience, s.lawful, s.food225);
                let e = &e.effective;
                let _ = write!(text, "\r\n【裝備計算基準：不是即時生命值】\r\n最大 HP {}／最大 MP {}／AC {}\r\n力量 {}／敏捷 {}／體質 {}／智力 {}／精神 {}／魅力 {}\r\n",
                    e.maximum_hp, e.maximum_mp, e.ac, e.strength, e.dexterity,
                    e.constitution, e.intelligence, e.wisdom, e.charisma);
            } else {
                text.push_str("本場未提供來源保存資料；不推定為零。\r\n");
            }
        }
        DetailsPage::Inventory => {
            text.push_str("【F5–F12 本場補給快捷格】\r\n先選取具備真實圖號的本場補給，再按 Ctrl+F5–F12 綁定；未選取時同組按鍵清除該格。關閉背包後在戰鬥視窗按對應 F 鍵或點格使用，仍以當下伺服器確認庫存與施放條件為準。F10／F11 在本場改屬快捷格；設定／聊天仍可使用畫面按鈕。來源裝備、材料與尚無權威圖號的箭矢不可綁定。\r\n\r\n");
            if let InventoryView::Selected(selected) = inventory {
                text.push_str(if selected.is_some() {
                    "【所選物品詳細資料】\r\n說明可捲動；更換物品會回到說明頂端，同一物品的確認更新保留捲動位置。\r\n\r\n"
                } else {
                    "【背包總覽】\r\n請在上方清單選擇物品，這裡會顯示該物品完整資料；不將其他物品說明混在一起。\r\n\r\n"
                });
            }
            text.push_str("【本場補給：使用後僅依伺服器確認扣量】\r\n上方選擇「本場補給」後按使用；選擇「本場箭矢」後按選用。選取清單本身不會操作；已選用箭矢、用完或條件不足的項目停用。數量與效果均等待伺服器確認，此頁不自動連續使用。\r\n");
            if let Some(s) = o.personal().supplies() {
                if s.stocks.0.is_empty() {
                    text.push_str("本場確認沒有補給。\r\n");
                }
                let _ = write!(text, "本場補給種類：{}\r\n", s.stocks.0.len());
                for s in s
                    .stocks
                    .0
                    .iter()
                    .filter(|s| inventory.includes(Key::Supply(s.item)))
                {
                    let _ = write!(
                        text,
                        "補給 #{}：剩餘 {}／開場 {}\r\n",
                        s.item, s.remaining, s.initial
                    );
                }
            } else {
                text.push_str("本場未提供補給，不代表來源背包為空。\r\n");
            }
            if let Some(a) = o.personal().ammunition() {
                let _ = write!(text, "本場箭矢種類：{}\r\n", a.stocks.len());
                for s in a
                    .stocks
                    .iter()
                    .filter(|s| inventory.includes(Key::Ammo(s.facts.item_id)))
                {
                    let _ = write!(
                        text,
                        "箭矢 #{}{}：剩餘 {}／開場 {}\r\n",
                        s.facts.item_id,
                        if s.facts.item_id == a.selected_item_id {
                            " [已選取]"
                        } else {
                            ""
                        },
                        s.remaining,
                        s.initial
                    );
                }
            }
            if let Some(m) = &d.personal.materials {
                let _ = write!(text, "\r\n【本場施法材料：僅依伺服器確認扣量】\r\n材料由技能自動消耗，不能在此使用或裝備；未命中也可能消耗，依伺服器回覆為準。\r\n材料種類：{}／確認版本 {}／施法消耗次數 {}\r\n",
                    m.stocks.0.len(), m.revision, m.uses);
                for s in m
                    .stocks
                    .0
                    .iter()
                    .filter(|s| inventory.includes(Key::Material(s.item)))
                {
                    let _ = write!(
                        text,
                        "材料 #{} · {}：剩餘 {}／開場 {}／已消耗 {}／單位重量 {}\r\n",
                        s.item,
                        super::battle_inventory::material_name(s.item, &label),
                        s.remaining,
                        s.initial,
                        s.initial - s.remaining,
                        s.unit_weight
                    );
                }
            }
            text.push_str("\r\n【來源保存背包／LiTo 裝備確認】\r\n本場補給與來源物品分開；支援新版裝備操作時，請在上方選擇物品實例後按裝備／卸下。沒有可用按鈕表示仍在等待確認或本場不支援，不會發送舊版 SET。此頁不交易或使用來源物品。\r\n");
            if let Some(e) = &d.personal.equipment {
                let _ = write!(
                    text,
                    "共 {} 筆來源物品／裝備版本 {}\r\n",
                    e.inventory.0.len(),
                    e.revision
                );
                for item in e
                    .inventory
                    .0
                    .iter()
                    .filter(|i| inventory.includes(Key::Source(i.object as u64)))
                {
                    let flag = match o.personal().equipped(item.object) {
                        Some(true) => "已裝備（LiTo 確認）",
                        Some(false) => "背包（LiTo 確認）",
                        None => "裝備狀態未提供",
                    };
                    let _ = write!(text, "\r\n[{}] {} · {:?}\r\n實例 {}／目錄 {}／數量 {}\r\n強化 {}／損壞 {}／祝福 {}／剩餘次數 {}\r\n已鑑定 {}／封印 {}／背包圖 {}／地面圖 {}／單位重量 {}\r\n原始 token：{}／來源保存裝備旗標 {}\r\n",
                        flag, label(&item.name.0), item.kind, item.object, item.catalog, item.quantity,
                        item.enchantment, item.durability, item.blessing, item.charge_count,
                        item.identified, item.sealed, item.inventory_graphic, item.ground_graphic, item.unit_weight,
                        item.name.0, item.equipped);
                    if let Some(l) = o.personal().lifetime(item.object) {
                        let _ = write!(text, "期限確認 {:?}／伺服器截止值 {}（Unix 毫秒；不依本機時鐘改穿戴狀態）\r\n", l.state, l.expires_at_millis);
                    }
                }
            } else {
                text.push_str("本場未提供來源背包；不是空背包。\r\n");
            }
        }
        DetailsPage::Skills => {
            text.push_str("【已學技能：選取文字不會施法】\r\n請在上方選技能，再按「選擇目標」或「施放」；也可選好可操作技能後按 Ctrl+F5–F12 綁定本場快捷格。關閉技能頁後按對應 F 鍵或點格；目標技能只進入選目標狀態，仍需左鍵選目標，右鍵可取消。瞬移、火風暴與強力加速術送出後等待伺服器確認。停用技能不會送出；下列來源模板不是即時耗魔或施放許可。\r\n");
            if let Some(lane) = d
                .actors
                .0
                .iter()
                .find(|a| a.entity == d.personal.entity)
                .and_then(|a| {
                    a.spells
                        .histories
                        .0
                        .iter()
                        .find(|h| h.lane == super::battle_payload::data::Lane::GreaterHaste)
                })
            {
                let _ = write!(text, "強力加速術（G／自身）：本場耗魔 {}；技能資格、間隔及狀態仍須伺服器確認。下列原始來源標記不是 v42 資格公告。\r\n", lane.ledger.cost);
            }
            if o.personal().skills().is_empty() {
                text.push_str("本場未提供技能列。\r\n");
            }
            for s in o.personal().skills() {
                let _ = write!(
                    text,
                    "#{} {}／模板 MP {}／模板間隔 {} ms／來源支援標記 {:?}\r\n原始 token：{}\r\n",
                    s.id,
                    label(&s.name.0),
                    s.mp_cost,
                    s.reuse_millis,
                    s.support,
                    s.name.0
                );
            }
        }
    }
    (!text.contains('\0') && text.encode_utf16().count() <= super::world_window::MAX_DETAILS_UTF16)
        .then_some(text)
}
