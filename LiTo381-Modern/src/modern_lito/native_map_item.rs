//! S_SHOW_MAP 0x6E：使用地圖道具後，伺服器回 `6E d(背包物件 ID) c(頁碼)`。
//!
//! 依據（唯讀靜態證據）：
//! - 正式 JAR E1498871 `Protocol.showMap(long,int)`：頁碼 1..18，`d` 是背包物件 ID。
//! - `ClientSession` 一般使用分支：`InventoryCatalog.Effect.MAP` 只在 BATTLE、非觀戰、HP>0
//!   時以 `NativeMapItemCatalog.requirePage(catalogId)` 查頁碼並送出；其餘情況只記錄、不回包。
//! - `NativeMapItemCatalog` 18 筆（catalog、page、icon、groundGfx、nameId），見 [`E149_ITEMS`]。
//! - 原版接收函式 `sub_4AFFB0`（PE 4271D92F…，只讀 bytes 未執行）：`"dc"` 解析後呼叫
//!   `447B70` 只確認背包裡有這個物件 ID（頁碼不參與比對），成功才把 `page-1` 寫入地圖面板
//!   `[0x75AC68]+0xC` 並以 `4EBEB0→4EBBD0` 開啟；物件不在背包時整包忽略。
//!
//! 本模組只保存「目前顯示哪一頁」。地圖頁圖片由哪個資源繪製尚未定位（見 issue-180 報告）。
use anyhow::{ensure, Context, Result};
use serde::Serialize;

pub const S_SHOW_MAP: u8 = 0x6e;
pub const PAGES: u8 = 18;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MapItem { pub catalog: u32, pub page: u8, pub icon: u16, pub ground_gfx: u16, pub name_id: &'static str }

/// 正式 E149 `NativeMapItemCatalog.ITEMS`（`<clinit>` 依序 18 筆）。注意 40373 是第 16 頁，
/// 40374–40388 依序是 1–15 頁，40389／40390 是 17／18 頁。
pub const E149_ITEMS: [MapItem; 18] = [
    MapItem { catalog: 40373, page: 16, icon: 1138, ground_gfx: 2982, name_id: "$1101" },
    MapItem { catalog: 40374, page: 1, icon: 479, ground_gfx: 1275, name_id: "$1102" },
    MapItem { catalog: 40375, page: 2, icon: 479, ground_gfx: 1275, name_id: "$1103" },
    MapItem { catalog: 40376, page: 3, icon: 479, ground_gfx: 1275, name_id: "$1104" },
    MapItem { catalog: 40377, page: 4, icon: 479, ground_gfx: 1275, name_id: "$1105" },
    MapItem { catalog: 40378, page: 5, icon: 479, ground_gfx: 1275, name_id: "$1106" },
    MapItem { catalog: 40379, page: 6, icon: 479, ground_gfx: 1275, name_id: "$1107" },
    MapItem { catalog: 40380, page: 7, icon: 479, ground_gfx: 1275, name_id: "$1108" },
    MapItem { catalog: 40381, page: 8, icon: 479, ground_gfx: 1275, name_id: "$1109" },
    MapItem { catalog: 40382, page: 9, icon: 479, ground_gfx: 1275, name_id: "$1188" },
    MapItem { catalog: 40383, page: 10, icon: 479, ground_gfx: 1275, name_id: "$1533" },
    MapItem { catalog: 40384, page: 11, icon: 479, ground_gfx: 1275, name_id: "$1534" },
    MapItem { catalog: 40385, page: 12, icon: 479, ground_gfx: 1275, name_id: "$1535" },
    MapItem { catalog: 40386, page: 13, icon: 479, ground_gfx: 1275, name_id: "$1607" },
    MapItem { catalog: 40387, page: 14, icon: 479, ground_gfx: 1275, name_id: "$1889" },
    MapItem { catalog: 40388, page: 15, icon: 479, ground_gfx: 1275, name_id: "$2203" },
    MapItem { catalog: 40389, page: 17, icon: 479, ground_gfx: 1275, name_id: "$2543" },
    MapItem { catalog: 40390, page: 18, icon: 479, ground_gfx: 1275, name_id: "$3616" },
];
pub fn by_page(page: u8) -> Option<&'static MapItem> { E149_ITEMS.iter().find(|m| m.page == page) }

/// `6E d(objid) c(page)`；objid 不可為 0，頁碼 1..=18，長度必須剛好 6。
pub fn parse(b: &[u8]) -> Result<Option<(u32, u8)>> {
    if b.first() != Some(&S_SHOW_MAP) { return Ok(None); }
    ensure!(b.len() == 6, "S_SHOW_MAP length");
    let id = u32::from_le_bytes(b[1..5].try_into()?);
    let page = b[5];
    ensure!(id != 0 && (1..=PAGES).contains(&page), "S_SHOW_MAP object or page");
    Ok(Some((id, page)))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Shown {
    pub item: u32,
    pub page: u8,
    pub owner: super::native_inventory::Owner,
    /// 背包物品名稱以正式 nameId 開頭時的對照 catalog；只供顯示，不參與接受判斷（原版也不比對）。
    pub catalog: Option<u32>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct View { pub shown: Option<Shown>, pub ignored: u64, pub revision: u64 }
impl View {
    /// 依原版 `447B70`：物件不在目前背包時忽略（計數，不報錯、不斷線）。
    pub fn accept(&mut self, inventory: &super::native_inventory::Inventory, b: &[u8]) -> Result<bool> {
        let Some((item, page)) = parse(b)? else { return Ok(false) };
        let (Some(owner), Some(entry)) = (inventory.owner.clone(), inventory.items.get(&item)) else {
            self.ignored = self.ignored.checked_add(1).context("map ignore count")?;
            return Ok(true);
        };
        let catalog = by_page(page).filter(|m| entry.name.starts_with(m.name_id)).map(|m| m.catalog);
        self.shown = Some(Shown { item, page, owner, catalog });
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }
    /// 物品被移除、換角色或換連線時關閉地圖頁。
    pub fn tick(&mut self, inventory: &super::native_inventory::Inventory) {
        if self.shown.as_ref().is_some_and(|s| inventory.owner.as_ref() != Some(&s.owner)
            || !inventory.items.contains_key(&s.item)) {
            self.close();
        }
    }
    pub fn close(&mut self) {
        if self.shown.take().is_some() { self.revision = self.revision.wrapping_add(1); }
    }
    pub fn label(&self) -> Option<String> {
        self.shown.as_ref().map(|s| format!("地圖第 {} 頁（共 {PAGES} 頁）", s.page))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::native_inventory::{Inventory, Item, Owner};
    use std::collections::BTreeSet;

    fn inventory(id: u32, name: &str) -> Inventory {
        let mut inv = Inventory::default();
        inv.owner = Some(Owner { connection: 4, selection: 1, character: "A".into() });
        inv.items.insert(id, Item { id, use_type: 0, charge: 0, icon: 479, blessing: 1, count: 1, identified: 1,
            name: name.into(), detail: vec![] });
        inv
    }

    #[test]
    fn e149_catalog_has_eighteen_items_covering_each_page_once() {
        let pages: BTreeSet<_> = E149_ITEMS.iter().map(|m| m.page).collect();
        assert_eq!(pages, (1..=18).collect());
        let ids: BTreeSet<_> = E149_ITEMS.iter().map(|m| m.catalog).collect();
        assert_eq!(ids, (40373..=40390).collect());
        assert_eq!(by_page(16).unwrap().catalog, 40373);
        assert_eq!(by_page(1).unwrap().catalog, 40374);
        assert_eq!(by_page(18).unwrap().name_id, "$3616");
    }

    #[test]
    fn show_map_parses_exact_dc_and_rejects_page_and_length_errors() {
        assert_eq!(parse(&[0x6e, 0x98, 0xba, 0xdc, 0xfe, 18]).unwrap(), Some((0xfedc_ba98, 18)));
        assert_eq!(parse(&[0x6e, 0xff, 0xff, 0xff, 0xff, 1]).unwrap(), Some((u32::MAX, 1)));
        for bad in [&[0x6e, 1, 0, 0, 0, 0][..], &[0x6e, 1, 0, 0, 0, 19], &[0x6e, 0, 0, 0, 0, 1],
            &[0x6e, 1, 0, 0, 0], &[0x6e, 1, 0, 0, 0, 1, 0]] {
            assert!(parse(bad).is_err(), "{bad:?}");
        }
        assert_eq!(parse(&[0x6f]).unwrap(), None);
    }

    #[test]
    fn view_follows_original_inventory_membership_and_closes_on_removal() {
        let mut inv = inventory(u32::MAX, "$1102");
        let mut view = View::default();
        assert!(view.accept(&inv, &[0x6e, 1, 0, 0, 0, 1]).unwrap());
        assert!(view.shown.is_none());
        assert_eq!(view.ignored, 1, "unknown object is ignored like 447B70");
        assert!(view.accept(&inv, &[0x6e, 0xff, 0xff, 0xff, 0xff, 1]).unwrap());
        let shown = view.shown.clone().unwrap();
        assert_eq!((shown.item, shown.page, shown.catalog), (u32::MAX, 1, Some(40374)));
        assert_eq!(view.label().unwrap(), "地圖第 1 頁（共 18 頁）");
        // The original does not compare page with the item; keep the server page.
        assert!(view.accept(&inv, &[0x6e, 0xff, 0xff, 0xff, 0xff, 7]).unwrap());
        assert_eq!(view.shown.as_ref().map(|s| (s.page, s.catalog)), Some((7, None)));
        view.tick(&inv);
        assert!(view.shown.is_some());
        inv.items.clear();
        view.tick(&inv);
        assert!(view.shown.is_none());
        let mut other = inventory(u32::MAX, "$1102");
        view.accept(&other, &[0x6e, 0xff, 0xff, 0xff, 0xff, 2]).unwrap();
        other.owner.as_mut().unwrap().selection = 2;
        view.tick(&other);
        assert!(view.shown.is_none(), "character reselection closes the page");
    }
}
