//! 三重矢（技能 132）扣箭：伺服器權威，客戶端只核對背包回包，不預扣。
//!
//! 依據（唯讀靜態證據，正式 JAR E1498871）：
//! - `ClientSession.useTripleArrowSpell`：要求已裝備弓（`SourceWeaponPhysicalStats.bow()`）、
//!   目標存在、MP 足夠（不足回 S_MESSAGE_CODE 278），受理後以 `hitCount()` 次呼叫
//!   `nativeTripleArrowAttempt`；每次都是 `CombatTripleArrow.enter`→一般 `attack()`→`leave`。
//! - 一般 `attack()` 每次射擊扣 1 支箭，所以成功施法最多扣 `hitCount`=3 支；之後的一般攻擊
//!   照常各扣 1 支（伺服器文件 `native-triple-arrow-policy-progress.zh-TW.md:7`）。
//! - 箭在正式目錄是 useType 254（0xFE）：40743 箭、40744 銀箭、40746 米索莉箭
//!   （`InventoryCatalog.<clinit>`）。
//!
//! 技能 132 的 C_USE_SPELL 送包屬 T022 批 3；本模組提供施法前檢查與施法後核對。
use super::native_inventory::Inventory;
use anyhow::{ensure, Result};
use serde::Serialize;
use std::collections::BTreeMap;

pub const SKILL: u16 = 132;
pub const HITS: u32 = 3;
pub const ARROW_USE_TYPE: u8 = 0xfe;

pub fn arrows(inventory: &Inventory) -> BTreeMap<u32, u32> {
    inventory.items.values().filter(|i| i.use_type == ARROW_USE_TYPE).map(|i| (i.id, i.count)).collect()
}
pub fn total(inventory: &Inventory) -> u64 { arrows(inventory).values().map(|&c| u64::from(c)).sum() }

/// 施法前快照：沒有任何箭時拒絕（伺服器的三次 attack 都不可能射出）。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Ledger { pub revision: u64, pub before: BTreeMap<u32, u32> }
impl Ledger {
    pub fn capture(inventory: &Inventory) -> Result<Self> {
        ensure!(inventory.owner.is_some(), "等待角色背包");
        let before = arrows(inventory);
        ensure!(before.values().any(|&c| c > 0), "背包沒有箭，無法施放三重矢");
        Ok(Self { revision: inventory.revision, before })
    }
    pub fn expected(&self) -> u64 {
        u64::from(HITS).min(self.before.values().map(|&c| u64::from(c)).sum())
    }
    /// 只根據伺服器送來的 S1A（數量）／S17（移除）計算；箭數增加（例如拾取）不算消耗。
    pub fn observe(&self, inventory: &Inventory) -> Consumption {
        let after = arrows(inventory);
        let mut consumed = 0u64;
        let mut removed = Vec::new();
        let mut gained = false;
        for (&id, &before) in &self.before {
            match after.get(&id) {
                None => { consumed += u64::from(before); removed.push(id); }
                Some(&now) if now < before => consumed += u64::from(before - now),
                Some(&now) if now > before => gained = true,
                _ => {}
            }
        }
        gained |= after.keys().any(|id| !self.before.contains_key(id));
        Consumption { consumed, removed, gained, updated: inventory.revision != self.revision }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Consumption { pub consumed: u64, pub removed: Vec<u32>, pub gained: bool, pub updated: bool }
impl Consumption {
    /// 單次三重矢的正式結果：扣 min(3, 原有箭數)，或被拒絕時完全不扣。
    pub fn matches_cast(&self, ledger: &Ledger) -> bool {
        !self.gained && (self.consumed == 0 || self.consumed == ledger.expected())
    }
    pub fn label(&self) -> String {
        if self.consumed == 0 { "伺服器尚未扣箭".into() } else { format!("伺服器扣除 {} 支箭", self.consumed) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::native_inventory::{Item, Owner};

    fn inv(rows: &[(u32, u8, u32)]) -> Inventory {
        let mut inv = Inventory::default();
        inv.owner = Some(Owner { connection: 1, selection: 1, character: "elf".into() });
        for &(id, use_type, count) in rows {
            inv.items.insert(id, Item { id, use_type, charge: 0, icon: 7, blessing: 1, count, identified: 1,
                name: "$61".into(), detail: vec![] });
        }
        inv
    }
    fn count(inv: &mut Inventory, id: u32, n: u32) {
        let mut b = vec![0x1a];
        b.extend(id.to_le_bytes());
        b.extend(n.to_le_bytes());
        inv.accept(1, &b).unwrap();
    }

    #[test]
    fn no_arrow_rejects_and_non_arrow_items_are_ignored() {
        assert!(Ledger::capture(&inv(&[(5, 0, 10)])).is_err());
        assert!(Ledger::capture(&Inventory::default()).is_err());
        let l = Ledger::capture(&inv(&[(5, 0, 10), (u32::MAX, 0xfe, 2)])).unwrap();
        assert_eq!(l.before.len(), 1);
        assert_eq!(l.expected(), 2, "fewer than three arrows caps the formal consumption");
    }

    #[test]
    fn three_arrows_per_cast_then_one_per_attack_only_from_server_receipts() {
        let mut i = inv(&[(0xfedc_ba98, 0xfe, 10)]);
        let l = Ledger::capture(&i).unwrap();
        assert_eq!(l.expected(), 3);
        let c = l.observe(&i);
        assert_eq!((c.consumed, c.updated), (0, false), "the client never pre-spends");
        assert!(c.matches_cast(&l), "a rejected cast consumes nothing");
        count(&mut i, 0xfedc_ba98, 7);
        let c = l.observe(&i);
        assert_eq!(c.consumed, 3);
        assert!(c.matches_cast(&l) && c.updated);
        assert_eq!(c.label(), "伺服器扣除 3 支箭");
        count(&mut i, 0xfedc_ba98, 6);
        assert!(!l.observe(&i).matches_cast(&l), "the following ordinary attack is a separate arrow");
        let next = Ledger::capture(&i).unwrap();
        count(&mut i, 0xfedc_ba98, 5);
        assert_eq!(next.observe(&i).consumed, 1);
    }

    #[test]
    fn removal_and_gain_are_classified_without_guessing() {
        let mut i = inv(&[(9, 0xfe, 3)]);
        let l = Ledger::capture(&i).unwrap();
        i.accept(1, &[0x17, 9, 0, 0, 0]).unwrap();
        let c = l.observe(&i);
        assert_eq!((c.consumed, c.removed.clone()), (3, vec![9]));
        assert!(c.matches_cast(&l));
        let mut j = inv(&[(9, 0xfe, 3)]);
        let l = Ledger::capture(&j).unwrap();
        count(&mut j, 9, 5);
        let c = l.observe(&j);
        assert!(c.gained && !c.matches_cast(&l));
    }
}
