//! 裝備與彈藥呈現（T052-1）：只讀伺服器送來的背包資料，不推測、不預扣。
//!
//! 依據（唯讀靜態證據，正式 JAR E1498871，`javap -p -c`）：
//! - `RuntimeInventory$Item.nativeViewName`：名稱＝`nativeLogName`，裝備中再接
//!   ` ($9)`（`definition.weapon()`）／` ($10)`（`light()`）／` ($117)`（`armor()`），
//!   之後才是 ` (次數)`（`maximumChargeCount() > 0` 且已鑑定）與 ` (數量)`（數量 > 1）。
//!   `wireItem()`（S16／S7C）與 `changeInventoryDescription`（S18／S6F）都用這個名稱。
//! - `RuntimeInventory.qualifiedRuntimeProjectile(weaponType)`：`weaponType == 10` 取
//!   useType 253（0xFD），其餘遠程武器取 254（0xFE）；依伺服器背包順序取第一疊
//!   「未裝備、數量 > 0、`InventoryCatalog.nativeProjectileStatus` 有值」的物品。
//!   客戶端不能指定用哪一疊，所以這裡只列出各疊與合計，不標示「使用中的那一疊」。
//! - `RuntimeInventory.weaponTypeForWalk`：walk action 20 → 4（弓），62 → 10（鐵手甲）。
//! - 缺彈藥時 `ClientSession.notifyMissingBattleProjectile` 送 `systemMessage(9, …)`。
use super::native_inventory::Inventory;
use serde::Serialize;

pub const ARROW_USE_TYPE: u8 = super::native_triple_arrow::ARROW_USE_TYPE;
pub const STING_USE_TYPE: u8 = 0xfd;
/// `ClientSession.notifyMissingBattleProjectile` 的原文（kind 9）。
pub const MISSING_AMMO_MESSAGE: &str = "沒有可用的箭矢或飛針，請先在商店購買。";

/// 伺服器在名稱上標示的裝備狀態。排序即清單內的顯示順序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Wear {
    Weapon,
    Armor,
    Light,
}

/// 去掉結尾一組 ` (十進位數字)`；不是這個形狀就回傳 `None`。
fn strip_number_suffix(name: &str) -> Option<&str> {
    let body = name.strip_suffix(')')?;
    let open = body.rfind(" (")?;
    let digits = &body[open + 2..];
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then(|| &name[..open])
}

/// 依 `nativeViewName` 的固定順序判讀：標記之後最多只有「次數」「數量」兩組數字。
/// 標記出現在其他位置（例如名稱中段）不算裝備中。
pub fn wear(name: &str) -> Option<Wear> {
    let mut rest = name;
    for _ in 0..3 {
        for (mark, wear) in [(" ($9)", Wear::Weapon), (" ($117)", Wear::Armor), (" ($10)", Wear::Light)] {
            if rest.ends_with(mark) {
                return Some(wear);
            }
        }
        rest = strip_number_suffix(rest)?;
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Ammo {
    Arrow,
    Sting,
}
impl Ammo {
    pub fn of_use_type(use_type: u8) -> Option<Self> {
        match use_type {
            ARROW_USE_TYPE => Some(Self::Arrow),
            STING_USE_TYPE => Some(Self::Sting),
            _ => None,
        }
    }
    /// 自己角色的 walk action 對應的彈藥種類；近戰姿勢回傳 `None`。
    pub fn for_stance(walk_action: u8) -> Option<Self> {
        match walk_action {
            20 => Some(Self::Arrow),
            62 => Some(Self::Sting),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Arrow => "箭",
            Self::Sting => "飛刀",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Stack {
    pub id: u32,
    pub count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// `(裝備狀態, 物件 ID)`，依 `Wear` 再依物件 ID 排序。
    pub worn: Vec<(Wear, u32)>,
    pub arrows: Vec<Stack>,
    pub stings: Vec<Stack>,
    /// 其餘物品的物件 ID（背包原順序）。
    pub others: Vec<u32>,
}
impl Summary {
    pub fn of(inventory: &Inventory) -> Self {
        let mut summary = Self::default();
        for item in inventory.items.values() {
            if let Some(wear) = wear(&item.name) {
                summary.worn.push((wear, item.id));
            } else if let Some(ammo) = Ammo::of_use_type(item.use_type) {
                let stack = Stack { id: item.id, count: item.count };
                match ammo {
                    Ammo::Arrow => summary.arrows.push(stack),
                    Ammo::Sting => summary.stings.push(stack),
                }
            } else {
                summary.others.push(item.id);
            }
        }
        summary.worn.sort();
        summary
    }
    pub fn stacks(&self, ammo: Ammo) -> &[Stack] {
        match ammo {
            Ammo::Arrow => &self.arrows,
            Ammo::Sting => &self.stings,
        }
    }
    pub fn total(&self, ammo: Ammo) -> u64 {
        self.stacks(ammo).iter().map(|s| u64::from(s.count)).sum()
    }
    pub fn worn_count(&self, wear: Wear) -> usize {
        self.worn.iter().filter(|(w, _)| *w == wear).count()
    }
    fn ammo_text(&self, ammo: Ammo) -> String {
        match self.stacks(ammo).len() {
            0 => format!("{} 0", ammo.label()),
            1 => format!("{} {}", ammo.label(), self.total(ammo)),
            stacks => format!("{} {}（{stacks} 疊）", ammo.label(), self.total(ammo)),
        }
    }
    fn worn_header(&self) -> String {
        if self.worn.is_empty() {
            return "【裝備中】沒有裝備中的物品".into();
        }
        format!(
            "【裝備中】武器 {}／防具 {}／照明 {}",
            self.worn_count(Wear::Weapon),
            self.worn_count(Wear::Armor),
            self.worn_count(Wear::Light)
        )
    }
    fn ammo_header(&self, stance: Option<u8>) -> String {
        let counts = format!("【彈藥】{}／{}", self.ammo_text(Ammo::Arrow), self.ammo_text(Ammo::Sting));
        match stance.and_then(Ammo::for_stance) {
            Some(ammo) => format!("{counts}；目前武器使用{}", ammo.label()),
            None => counts,
        }
    }
    /// 背包清單的分組：裝備中 → 彈藥 → 其他。空背包不產生任何列。
    pub fn rows(&self, stance: Option<u8>) -> Vec<Row> {
        if self.worn.is_empty() && self.arrows.is_empty() && self.stings.is_empty() && self.others.is_empty() {
            return Vec::new();
        }
        let mut rows = vec![Row::Header(self.worn_header())];
        rows.extend(self.worn.iter().map(|(_, id)| Row::Item(*id)));
        rows.push(Row::Header(self.ammo_header(stance)));
        rows.extend(self.arrows.iter().chain(&self.stings).map(|s| Row::Item(s.id)));
        if !self.others.is_empty() {
            rows.push(Row::Header(format!("【其他物品】{} 項", self.others.len())));
            rows.extend(self.others.iter().map(|id| Row::Item(*id)));
        }
        rows
    }
    /// 世界 HUD 的彈藥文字；自己不是遠程姿勢時不顯示。數量只來自伺服器背包封包。
    pub fn hud(&self, stance: u8) -> Option<String> {
        Ammo::for_stance(stance).map(|ammo| self.ammo_text(ammo))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Row {
    /// 分組標題，不對應任何物品。
    Header(String),
    Item(u32),
}

/// 伺服器的缺彈藥訊息：收到後顯示到背包下一次變動（例如買到箭）為止，
/// 不會因為這則訊息一直留在 `last_system_message` 而永久顯示。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MissingAmmoNotice {
    seen: Option<(u64, u64)>,
}
impl MissingAmmoNotice {
    /// `message_count`／`message` 為世界的系統訊息計數與最後一則；`revision` 為背包版本。
    pub fn observe(&mut self, message_count: u64, message: Option<&(u8, String)>, revision: u64) -> bool {
        if !message.is_some_and(|(kind, text)| *kind == 9 && text == MISSING_AMMO_MESSAGE) {
            self.seen = None;
            return false;
        }
        if self.seen.map(|(count, _)| count) != Some(message_count) {
            self.seen = Some((message_count, revision));
        }
        self.seen.is_some_and(|(_, seen)| seen == revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::native_inventory::{Item, Owner};

    fn item(id: u32, use_type: u8, count: u32, name: &str) -> Item {
        Item { id, use_type, charge: 0, icon: 1, blessing: 1, count, identified: 1, name: name.into(), detail: vec![] }
    }
    fn inv(rows: &[(u32, u8, u32, &str)]) -> Inventory {
        let mut inv = Inventory::default();
        inv.owner = Some(Owner { connection: 1, selection: 1, character: "elf".into() });
        for &(id, use_type, count, name) in rows {
            inv.items.insert(id, item(id, use_type, count, name));
        }
        inv
    }

    #[test]
    fn wear_follows_the_formal_view_name_order() {
        assert_eq!(wear("+7 $41 ($9)"), Some(Wear::Weapon));
        assert_eq!(wear("$20 ($117)"), Some(Wear::Armor));
        assert_eq!(wear("$100 ($10)"), Some(Wear::Light));
        // 標記之後依序是次數、數量。
        assert_eq!(wear("+0 $41 ($9) (3)"), Some(Wear::Weapon));
        assert_eq!(wear("$100 ($10) (12) (2)"), Some(Wear::Light));
        // 未裝備、只有次數／數量。
        assert_eq!(wear("$41"), None);
        assert_eq!(wear("$40 (300)"), None);
        assert_eq!(wear("松木魔杖 (7) (2)"), None);
        assert_eq!(wear(""), None);
        // 標記不在正式位置：不當成裝備中。
        assert_eq!(wear("($9)"), None, "標記前必須有空白");
        assert_eq!(wear("$41 ($9) x"), None);
        assert_eq!(wear("$41 ($9)(3)"), None);
        assert_eq!(wear("$41 ($9) (3) (2) (1)"), None, "標記之後最多兩組數字");
        assert_eq!(wear("$41 ($9) ()"), None);
        assert_eq!(wear("$41 ($9) (a)"), None);
        assert_eq!(wear("$41 ($11)"), None);
        assert_eq!(wear("$41 ($1170)"), None);
    }

    #[test]
    fn ammo_kinds_match_the_formal_use_types_and_stances() {
        assert_eq!(Ammo::of_use_type(0xfe), Some(Ammo::Arrow));
        assert_eq!(Ammo::of_use_type(0xfd), Some(Ammo::Sting));
        for other in [0u8, 1, 2, 0xfc, 0xff] {
            assert_eq!(Ammo::of_use_type(other), None);
        }
        assert_eq!(Ammo::for_stance(20), Some(Ammo::Arrow));
        assert_eq!(Ammo::for_stance(62), Some(Ammo::Sting));
        // E149 weaponTypeForWalk 的其餘姿勢都是近戰。
        for melee in [0u8, 4, 11, 24, 40, 46, 50, 54, 58, 83] {
            assert_eq!(Ammo::for_stance(melee), None);
        }
    }

    #[test]
    fn summary_groups_worn_items_ammo_and_the_rest_without_guessing() {
        let inventory = inv(&[
            (10, 0, 5, "$26 (5)"),
            (11, 2, 1, "$20 ($117)"),
            (12, 1, 1, "+7 $41 ($9)"),
            (13, 0xfe, 300, "$40 (300)"),
            (0xfedc_ba98, 0xfe, 7, "$42 (7)"),
            (15, 0xfd, 1, "$50"),
            (16, 22, 1, "$100 ($10)"),
            (17, 2, 1, "$21 ($117)"),
            (18, 1, 1, "$43"),
        ]);
        let s = Summary::of(&inventory);
        assert_eq!(
            s.worn,
            vec![(Wear::Weapon, 12), (Wear::Armor, 11), (Wear::Armor, 17), (Wear::Light, 16)]
        );
        assert_eq!(s.arrows, vec![Stack { id: 13, count: 300 }, Stack { id: 0xfedc_ba98, count: 7 }]);
        assert_eq!(s.stings, vec![Stack { id: 15, count: 1 }]);
        assert_eq!(s.others, vec![10, 18]);
        assert_eq!((s.total(Ammo::Arrow), s.total(Ammo::Sting)), (307, 1));
        assert_eq!(
            s.rows(None),
            vec![
                Row::Header("【裝備中】武器 1／防具 2／照明 1".into()),
                Row::Item(12),
                Row::Item(11),
                Row::Item(17),
                Row::Item(16),
                Row::Header("【彈藥】箭 307（2 疊）／飛刀 1".into()),
                Row::Item(13),
                Row::Item(0xfedc_ba98),
                Row::Item(15),
                Row::Header("【其他物品】2 項".into()),
                Row::Item(10),
                Row::Item(18),
            ]
        );
        // 每件物品剛好出現一次。
        let listed = s.rows(Some(20)).iter().filter(|r| matches!(r, Row::Item(_))).count();
        assert_eq!(listed, inventory.items.len());
    }

    #[test]
    fn headers_and_hud_follow_the_own_stance() {
        let s = Summary::of(&inv(&[(1, 0xfe, 20, "$40 (20)")]));
        assert_eq!(
            s.rows(Some(20)),
            vec![
                Row::Header("【裝備中】沒有裝備中的物品".into()),
                Row::Header("【彈藥】箭 20／飛刀 0；目前武器使用箭".into()),
                Row::Item(1),
            ]
        );
        assert_eq!(s.rows(Some(62))[1], Row::Header("【彈藥】箭 20／飛刀 0；目前武器使用飛刀".into()));
        assert_eq!(s.rows(Some(4))[1], Row::Header("【彈藥】箭 20／飛刀 0".into()));
        assert_eq!(s.hud(20).as_deref(), Some("箭 20"));
        assert_eq!(s.hud(62).as_deref(), Some("飛刀 0"));
        assert_eq!(s.hud(4), None);
        assert!(Summary::of(&Inventory::default()).rows(Some(20)).is_empty());
        // 大數量不溢位。
        let big = Summary::of(&inv(&[(1, 0xfe, u32::MAX, "a"), (2, 0xfe, u32::MAX, "b")]));
        assert_eq!(big.total(Ammo::Arrow), 2 * u64::from(u32::MAX));
    }

    #[test]
    fn ammo_counts_change_only_with_server_inventory_packets() {
        let mut inventory = inv(&[(9, 0xfe, 20, "$40 (20)"), (3, 1, 1, "$41 ($9)")]);
        assert_eq!(Summary::of(&inventory).hud(20).as_deref(), Some("箭 20"));
        let mut count = vec![0x1a];
        count.extend(9u32.to_le_bytes());
        count.extend(19u32.to_le_bytes());
        inventory.accept(1, &count).unwrap();
        assert_eq!(Summary::of(&inventory).hud(20).as_deref(), Some("箭 19"));
        // S18 卸下武器：名稱不再帶標記。
        let mut name = vec![0x18];
        name.extend(3u32.to_le_bytes());
        name.extend(b"$41\0");
        inventory.accept(1, &name).unwrap();
        let s = Summary::of(&inventory);
        assert!(s.worn.is_empty());
        assert_eq!(s.others, vec![3]);
        // S17 移除最後一疊。
        let mut remove = vec![0x17];
        remove.extend(9u32.to_le_bytes());
        inventory.accept(1, &remove).unwrap();
        assert_eq!(Summary::of(&inventory).hud(20).as_deref(), Some("箭 0"));
    }

    #[test]
    fn missing_ammo_notice_lasts_until_the_inventory_changes() {
        let missing = (9u8, MISSING_AMMO_MESSAGE.to_owned());
        let other = (9u8, "戰鬥中無法退出，請停止攻擊並等待 3 秒".to_owned());
        let wrong_kind = (2u8, MISSING_AMMO_MESSAGE.to_owned());
        let mut notice = MissingAmmoNotice::default();
        assert!(!notice.observe(0, None, 5));
        assert!(!notice.observe(1, Some(&other), 5));
        assert!(!notice.observe(2, Some(&wrong_kind), 5));
        assert!(notice.observe(3, Some(&missing), 5));
        assert!(notice.observe(3, Some(&missing), 5), "重複取樣維持顯示");
        assert!(!notice.observe(3, Some(&missing), 6), "背包變動後不再顯示舊訊息");
        assert!(!notice.observe(3, Some(&missing), 7));
        assert!(notice.observe(4, Some(&missing), 7), "伺服器再次拒絕時重新顯示");
        assert!(!notice.observe(5, Some(&other), 7));
        assert!(notice.observe(6, Some(&missing), 7));
    }
}
