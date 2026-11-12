//! rental-source 服務模式的客戶端契約（T055-2，G06.3）。
//!
//! 三件事都只做「客戶端不送出／不接受原生流程不可能產生的東西」，權威仍在伺服器：
//! - C_CA 配裝選擇：依所選角色 S_CB 職業位元組限制可選數量；
//! - S_FB 直連房目錄：進村交接期間目錄必須仍提供已快取的房間 id，加入結果必須是 mode 6 租裝村；
//! - 租裝村（map500／560）與旅館（map501–503）之間的 S_PORTAL 必須屬於伺服器會產生的三種入口之一。
//!
//! 唯讀參考 lito-launcher-server `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`：
//! `ClientSession.selectRentalItemSet`（職業 0/1/4→3 組、2/3→2 組、其餘 0 組）、
//! `ClientSession.joinRentalTown`、`sendLitoServerList`、`enterRentalTownPortal`
//! （C73 只接受同圖目的地）、`beginRentalTownMapTransition`（只允許 500→旅館與旅館→500）、
//! `NativeLitoRentalHotelCatalog`（501..503、RETURN_MAP_ID=500）、`ServerConfig`（租裝村只能是 500 或 560）。
use anyhow::{bail, ensure, Context, Result};

/// `lito.rental.town.map` 只接受原生 map500（Lodun）或 map560（Lenoa）。
pub const TOWN_MAPS: [u16; 2] = [500, 560];
/// 租裝旅館房間地圖。
pub const HOTEL_MAPS: std::ops::RangeInclusive<u16> = 501..=503;
/// 旅館只能由 map500 進入，也只會返回 map500。
pub const HOTEL_RETURN_MAP: u16 = 500;
/// 租裝村加入結果的房間模式。
pub const TOWN_MODE: u8 = 6;

/// 所選職業（S_CB 第一個數值位元組）可用的租裝配裝數；0 表示此職業不能租裝。
pub fn item_set_choices(class: u8) -> u8 {
    match class {
        0 | 1 | 4 => 3,
        2 | 3 => 2,
        _ => 0,
    }
}

/// C_CA(index) 送出前的本地檢查；不合法時不送出封包。
pub fn admit_item_set(class: Option<u8>, index: u8) -> Result<()> {
    let class = class.context("找不到所選角色的職業資料")?;
    let choices = item_set_choices(class);
    ensure!(choices > 0, "此職業沒有可用的租裝配裝");
    ensure!(index < choices, "配裝選項超出此職業範圍");
    Ok(())
}

/// S_FB 在進村交接期間更新時，仍須提供已快取的直連房 id。
pub fn directory_retains(directory: &super::native_services::Directory, room: u32) -> bool {
    directory.rooms.iter().any(|r| r.id == room)
}

/// 租裝村 C_CF 的加入結果必須是同一房間 id 的 mode 6 租裝村。
pub fn admit_join(room: &super::native_room::Room, requested: u32) -> Result<()> {
    ensure!(room.id == requested, "租裝村加入結果房間不符");
    ensure!(room.mode == TOWN_MODE, "租裝服務加入結果不是租裝村模式");
    ensure!(TOWN_MAPS.contains(&room.map_id), "租裝村加入結果地圖不符");
    Ok(())
}

/// 租裝村內伺服器會產生的地圖轉換種類。
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum TownTransition {
    /// C73 原圖 tag100 入口；伺服器只回同圖目的地（也接受等待逾時後的晚到回覆）。
    GroundPortal,
    /// NPC 旅館入住：map500 → 501..503。
    HotelEnter,
    /// 旅館返回（手動、租約失效或到期）：501..503 → map500。
    HotelReturn,
}

/// 分類一筆 mode 6 S_PORTAL；不屬於任何種類就拒絕（不送 C08）。
pub fn classify(
    current: Option<u16>,
    destination: super::native_portal::Destination,
    ground_pending: bool,
) -> Result<TownTransition> {
    let current = current.context("租裝村入口缺少目前地圖")?;
    let target = destination.map;
    if TOWN_MAPS.contains(&current) {
        if target == current {
            return Ok(TownTransition::GroundPortal);
        }
        ensure!(!ground_pending, "原圖入口回覆不是同圖目的地");
        if current == HOTEL_RETURN_MAP && HOTEL_MAPS.contains(&target) {
            return Ok(TownTransition::HotelEnter);
        }
        bail!("租裝村入口目的地不符");
    }
    if HOTEL_MAPS.contains(&current) {
        ensure!(target == HOTEL_RETURN_MAP, "旅館只會返回租裝村");
        return Ok(TownTransition::HotelReturn);
    }
    bail!("目前地圖不是租裝村或旅館")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::native_portal::Destination;

    fn to(map: u16) -> Destination {
        Destination { map, x: 32788, y: 32726 }
    }

    #[test]
    fn item_set_choices_follow_server_class_table() {
        let expected = [3, 3, 2, 2, 3, 0, 0, 0];
        for (class, n) in expected.iter().enumerate() {
            assert_eq!(item_set_choices(class as u8), *n, "class {class}");
        }
        assert!(admit_item_set(Some(1), 2).is_ok());
        assert!(admit_item_set(Some(1), 3).is_err());
        assert!(admit_item_set(Some(2), 1).is_ok());
        assert!(admit_item_set(Some(3), 2).is_err());
        assert!(admit_item_set(Some(6), 0).is_err());
        assert!(admit_item_set(None, 0).is_err());
    }

    #[test]
    fn town_transitions_accept_only_server_produced_kinds() {
        use TownTransition::*;
        // 地面入口：兩個租裝村都只會同圖；晚到回覆（pending 已清除）同樣接受。
        for map in TOWN_MAPS {
            assert_eq!(classify(Some(map), to(map), true).unwrap(), GroundPortal);
            assert_eq!(classify(Some(map), to(map), false).unwrap(), GroundPortal);
        }
        // 旅館入住只能從 map500。
        for hotel in HOTEL_MAPS {
            assert_eq!(classify(Some(500), to(hotel), false).unwrap(), HotelEnter);
            assert!(classify(Some(500), to(hotel), true).is_err());
            assert!(classify(Some(560), to(hotel), false).is_err());
            assert_eq!(classify(Some(hotel), to(500), false).unwrap(), HotelReturn);
            assert!(classify(Some(hotel), to(560), false).is_err());
            assert!(classify(Some(hotel), to(hotel), false).is_err());
        }
        // 跨村、非租裝地圖、未知目前地圖一律拒絕。
        assert!(classify(Some(500), to(560), false).is_err());
        assert!(classify(Some(560), to(500), false).is_err());
        assert!(classify(Some(500), to(4), false).is_err());
        assert!(classify(Some(4), to(4), false).is_err());
        assert!(classify(None, to(500), false).is_err());
    }
}
