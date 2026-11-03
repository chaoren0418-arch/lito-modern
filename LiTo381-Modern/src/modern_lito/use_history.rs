//! Historical self-resource use, independent of current equipment. Not yet a
//! network capability or target damage proof; old wire validators stay strict.
use super::{equipment_transition::Vitals, item_protocol, skill_spending::Ledger};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchor {
    pub equipment_revision: i64,
    pub life: i64,
    pub vitals: Vitals,
    pub x: i32,
    pub y: i32,
}
impl Anchor {
    pub fn valid(self) -> bool {
        self.equipment_revision >= 0 && self.life >= 1 && self.vitals.valid() && self.vitals.hp > 0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cast {
    pub revision: i64,
    pub sequence: i64,
    pub before: Anchor,
    pub cost: i32,
    pub mp_after: i64,
}
impl Cast {
    pub fn valid(self) -> bool {
        self.before.valid()
            && self.revision >= 2
            && self.sequence >= 1
            && self.cost >= 0
            && i64::from(self.cost) <= self.before.vitals.mp
            && self.mp_after == self.before.vitals.mp - i64::from(self.cost)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spell {
    pub current: Ledger,
    pub last: Option<Cast>,
}
impl Spell {
    pub fn valid(self) -> bool {
        if !self.current.valid() || (self.current.casts == 0) != self.last.is_none() {
            return false;
        }
        self.last.map_or(true, |last| {
            last.valid()
                && last.revision == self.current.last_cast_revision
                && last.sequence == self.current.last_cast_sequence
                && i64::from(last.cost) <= self.current.spent
                && (self.current.casts != 1 || i64::from(last.cost) == self.current.spent)
                && last.before.equipment_revision <= self.current.equipment_revision
                && if last.before.equipment_revision == self.current.equipment_revision {
                    last.revision > self.current.profile_revision && last.cost == self.current.cost
                } else {
                    last.revision < self.current.profile_revision
                }
        })
    }
    pub fn initial(gear: i64, revision: i64, cost: i32) -> Option<Self> {
        Some(Self {
            current: Ledger::initial(gear, revision, cost)?,
            last: None,
        })
    }
    pub fn cast(self, revision: i64, sequence: i64, before: Anchor) -> Option<Self> {
        if !self.valid()
            || !before.valid()
            || before.equipment_revision != self.current.equipment_revision
            || self.last.is_some_and(|last| before.life < last.before.life)
        {
            return None;
        }
        let value = Self {
            current: self.current.cast(revision, sequence)?,
            last: Some(Cast {
                revision,
                sequence,
                before,
                cost: self.current.cost,
                mp_after: before.vitals.mp.checked_sub(i64::from(self.current.cost))?,
            }),
        };
        value.valid().then_some(value)
    }
    pub fn reprice(self, gear: i64, revision: i64, cost: i32) -> Option<Self> {
        if !self.valid() {
            return None;
        }
        let value = Self {
            current: self.current.reprice(gear, revision, cost)?,
            last: self.last,
        };
        value.valid().then_some(value)
    }
    pub fn follows(self, after: Self) -> bool {
        self.valid()
            && after.valid()
            && (self == after
                || after.last.is_some_and(|last| {
                    self.cast(last.revision, last.sequence, last.before) == Some(after)
                })
                || self.reprice(
                    after.current.equipment_revision,
                    after.current.profile_revision,
                    after.current.cost,
                ) == Some(after))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Use {
    pub revision: i64,
    pub sequence: i64,
    pub item_id: i64,
    pub before: Anchor,
    pub fixed_recovery: i64,
    pub rolled_base: i64,
    pub restored: i64,
    pub hp_after: i64,
}
impl Use {
    pub fn valid(self) -> bool {
        if !self.before.valid()
            || self.revision < 2
            || self.sequence < 1
            || self.fixed_recovery < 0
            || self.restored < 0
        {
            return false;
        }
        let expected = if let Some(kind) = item_protocol::healing(self.item_id) {
            if !(kind.minimum..=kind.maximum).contains(&self.rolled_base) {
                return false;
            }
            let Some(raw) = self.rolled_base.checked_add(self.fixed_recovery) else {
                return false;
            };
            raw.min(self.before.vitals.maximum_hp - self.before.vitals.hp)
        } else {
            if !item_protocol::supported(self.item_id)
                || self.rolled_base != 0
                || self.fixed_recovery != 0
            {
                return false;
            }
            0
        };
        self.restored == expected
            && self.before.vitals.hp.checked_add(expected) == Some(self.hp_after)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Items {
    pub equipment_revision: i64,
    pub profile_revision: i64,
    pub fixed_recovery: i64,
    pub uses: i64,
    pub restored: i64,
    pub last: Option<Use>,
}
impl Items {
    pub fn valid(self) -> bool {
        if self.equipment_revision < 0
            || self.profile_revision < 1
            || self.fixed_recovery < 0
            || self.uses < 0
            || self.restored < 0
            || (self.uses == 0) != self.last.is_none()
            || self.uses == 0 && self.restored != 0
        {
            return false;
        }
        self.last.map_or(true, |last| {
            last.valid()
                && self.uses <= last.sequence
                && self.uses < last.revision
                && self.restored >= last.restored
                && last.before.equipment_revision <= self.equipment_revision
                && (self.uses != 1 || self.restored == last.restored)
                && if last.before.equipment_revision == self.equipment_revision {
                    last.revision > self.profile_revision
                        && (item_protocol::healing(last.item_id).is_none()
                            || last.fixed_recovery == self.fixed_recovery)
                } else {
                    last.revision < self.profile_revision
                }
        })
    }
    pub fn initial(gear: i64, revision: i64, fixed: i64) -> Option<Self> {
        let value = Self {
            equipment_revision: gear,
            profile_revision: revision,
            fixed_recovery: fixed,
            uses: 0,
            restored: 0,
            last: None,
        };
        value.valid().then_some(value)
    }
    pub fn use_item(
        self,
        revision: i64,
        sequence: i64,
        item_id: i64,
        before: Anchor,
        rolled_base: i64,
    ) -> Option<Self> {
        if !self.valid()
            || !before.valid()
            || before.equipment_revision != self.equipment_revision
            || revision <= self.profile_revision
            || self.last.is_some_and(|last| {
                revision <= last.revision
                    || sequence <= last.sequence
                    || before.life < last.before.life
            })
        {
            return None;
        }
        let potion = item_protocol::healing(item_id).is_some();
        let fixed_recovery = if potion { self.fixed_recovery } else { 0 };
        let restored = if potion {
            rolled_base
                .checked_add(fixed_recovery)?
                .min(before.vitals.maximum_hp - before.vitals.hp)
        } else {
            0
        };
        let last = Use {
            revision,
            sequence,
            item_id,
            before,
            fixed_recovery,
            rolled_base,
            restored,
            hp_after: before.vitals.hp.checked_add(restored)?,
        };
        let value = Self {
            uses: self.uses.checked_add(1)?,
            restored: self.restored.checked_add(restored)?,
            last: Some(last),
            ..self
        };
        value.valid().then_some(value)
    }
    pub fn reprofile(self, gear: i64, revision: i64, fixed: i64) -> Option<Self> {
        if !self.valid()
            || self.equipment_revision.checked_add(1)? != gear
            || revision <= self.profile_revision
            || self.last.is_some_and(|last| revision <= last.revision)
        {
            return None;
        }
        let value = Self {
            equipment_revision: gear,
            profile_revision: revision,
            fixed_recovery: fixed,
            ..self
        };
        value.valid().then_some(value)
    }
    pub fn follows(self, after: Self) -> bool {
        self.valid()
            && after.valid()
            && (self == after
                || after.last.is_some_and(|last| {
                    self.use_item(
                        last.revision,
                        last.sequence,
                        last.item_id,
                        last.before,
                        last.rolled_base,
                    ) == Some(after)
                })
                || self.reprofile(
                    after.equipment_revision,
                    after.profile_revision,
                    after.fixed_recovery,
                ) == Some(after))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn anchor(gear: i64, life: i64, hp: i64, cap: i64, mp: i64) -> Anchor {
        Anchor {
            equipment_revision: gear,
            life,
            vitals: Vitals {
                hp,
                maximum_hp: cap,
                mp,
                maximum_mp: i64::MAX,
            },
            x: 32768,
            y: 32770,
        }
    }
    #[test]
    fn repricing_retains_old_cost_life_and_capacities() {
        let first = Spell::initial(0, 1, 70)
            .unwrap()
            .cast(2, 1, anchor(0, 1, 99, 100, 100))
            .unwrap();
        let changed = first.reprice(1, 3, 0).unwrap().reprice(2, 4, 59).unwrap();
        assert_eq!(first.last, changed.last);
        assert_eq!(changed.current.spent, 70);
        let next = changed.cast(5, 4, anchor(2, 2, 10, 10, 59)).unwrap();
        assert_eq!(next.current.spent, 129);
        assert_eq!(next.last.unwrap().mp_after, 0);
        assert!(changed.follows(next));
        assert!(!first.follows(changed));
        assert!(next.cast(6, 5, anchor(2, 1, 1, 1, 100)).is_none());
        assert!(changed.cast(5, 4, anchor(1, 2, 10, 10, 59)).is_none());
    }
    #[test]
    fn free_cast_wide_mp_and_forged_raw_structs_fail_closed() {
        let first = Spell::initial(0, 1, 0)
            .unwrap()
            .cast(2, 1, anchor(0, 1, 1, 1, i64::MAX))
            .unwrap();
        assert_eq!(first.current.casts, 1);
        assert_eq!(first.current.spent, 0);
        assert_eq!(first.last.unwrap().mp_after, i64::MAX);
        assert!(Spell::initial(0, 1, 70)
            .unwrap()
            .cast(2, 1, anchor(0, 1, 1, 1, 69))
            .is_none());
        let mut forged = first;
        forged.last.as_mut().unwrap().mp_after = 1;
        assert!(!forged.valid());
        assert!(!first.follows(forged));
        assert!(forged.reprice(1, 3, 1).is_none());
        assert!(forged.cast(3, 2, anchor(0, 1, 1, 1, 10)).is_none());
        let mut bad_anchor = anchor(0, 1, 1, 1, 0);
        bad_anchor.vitals.maximum_hp = -1;
        assert!(first.cast(3, 2, bad_anchor).is_none());
    }
    #[test]
    fn potion_uses_event_cap_roll_and_equipment_bonus_not_current_values() {
        let first = Items::initial(0, 1, 3_000_000_000)
            .unwrap()
            .use_item(2, 1, 40010, anchor(0, 1, 95, 100, 50), 20)
            .unwrap();
        assert_eq!(first.last.unwrap().restored, 5);
        let changed = first.reprofile(1, 3, 0).unwrap();
        assert_eq!(first.last, changed.last);
        let next = changed
            .use_item(4, 3, 40011, anchor(1, 2, 1, 500, 20), 30)
            .unwrap();
        assert_eq!(next.restored, 35);
        assert_eq!(next.last.unwrap().hp_after, 31);
        assert!(first.follows(changed) && changed.follows(next));
        assert!(!first.follows(next));
        assert!(changed
            .use_item(4, 2, 40011, anchor(0, 2, 1, 500, 20), 30)
            .is_none());
    }
    #[test]
    fn full_hp_timed_and_overflow_never_mint_recovery() {
        let initial = Items::initial(0, 1, 3).unwrap();
        let full = initial
            .use_item(2, 1, 40010, anchor(0, 1, 100, 100, 20), 10)
            .unwrap();
        let timed = full
            .use_item(3, 2, 40013, anchor(0, 1, 90, 100, 20), 0)
            .unwrap();
        assert_eq!(
            (
                timed.uses,
                timed.restored,
                timed.last.unwrap().fixed_recovery,
                timed.last.unwrap().hp_after
            ),
            (2, 0, 0, 90)
        );
        assert!(initial
            .use_item(2, 1, 40010, anchor(0, 1, 1, 100, 0), 9)
            .is_none());
        assert!(initial
            .use_item(2, 1, 40013, anchor(0, 1, 1, 100, 0), 10)
            .is_none());
        assert!(initial
            .use_item(2, 1, 999, anchor(0, 1, 1, 100, 0), 0)
            .is_none());
        assert!(Items::initial(0, 1, i64::MAX)
            .unwrap()
            .use_item(2, 1, 40010, anchor(0, 1, 100, 100, 0), 10)
            .is_none());
        let mut forged = timed;
        forged.last.as_mut().unwrap().restored = 1;
        assert!(!forged.valid());
        assert!(forged.reprofile(1, 4, 0).is_none());
        assert!(forged
            .use_item(4, 3, 40010, anchor(0, 1, 1, 100, 0), 10)
            .is_none());
    }
    #[test]
    fn individually_valid_but_rewritten_old_receipts_do_not_follow() {
        let first = Spell::initial(0, 1, 10)
            .unwrap()
            .cast(2, 1, anchor(0, 1, 100, 100, 100))
            .unwrap();
        let changed = first.reprice(1, 3, 9).unwrap();
        let mut fake = changed;
        fake.last = Some(Cast {
            revision: 2,
            sequence: 1,
            before: anchor(0, 1, 1, 1, 20),
            cost: 10,
            mp_after: 10,
        });
        assert!(fake.valid());
        assert!(!first.follows(fake));
        assert!(!changed.follows(fake));
        let item = Items::initial(0, 1, 0)
            .unwrap()
            .use_item(2, 1, 40010, anchor(0, 1, 1, 100, 20), 10)
            .unwrap();
        let repriced = item.reprofile(1, 3, 9).unwrap();
        let mut forged = repriced;
        forged.last = Some(Use {
            revision: 2,
            sequence: 1,
            item_id: 40010,
            before: anchor(0, 1, 20, 100, 20),
            fixed_recovery: 0,
            rolled_base: 10,
            restored: 10,
            hp_after: 30,
        });
        assert!(forged.valid());
        assert!(!item.follows(forged));
        assert!(!repriced.follows(forged));
    }
    #[test]
    fn fixed_java_rust_corpus_preserves_receipts_across_every_reprice() {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        let mut revision = 1;
        let mut spell = Spell::initial(0, 1, 70).unwrap();
        let mut item = Items::initial(0, 1, 3_000_000_000).unwrap();
        for i in 0..256_i64 {
            let a = anchor(i, 1 + i / 17, 1 + i % 80, 100 + i, i64::MAX - i);
            revision += 1;
            spell = spell.cast(revision, i * 2 + 1, a).unwrap();
            item = item
                .use_item(revision, i * 2 + 1, 40010, a, 10 + i % 11)
                .unwrap();
            let last_spell = spell.last.unwrap();
            let last_item = item.last.unwrap();
            revision += 1;
            spell = spell
                .reprice(i + 1, revision, if i % 3 == 0 { 0 } else { 59 })
                .unwrap();
            item = item
                .reprofile(i + 1, revision, if i % 2 == 0 { 0 } else { 3_000_000_000 })
                .unwrap();
            assert_eq!(spell.last, Some(last_spell));
            assert_eq!(item.last, Some(last_item));
            for value in [
                spell.current.equipment_revision,
                spell.current.profile_revision,
                i64::from(spell.current.cost),
                spell.current.casts,
                spell.current.spent,
                i64::from(last_spell.cost),
                last_spell.mp_after,
                last_spell.before.life,
                last_spell.before.vitals.maximum_hp,
                item.equipment_revision,
                item.profile_revision,
                item.fixed_recovery,
                item.uses,
                item.restored,
                last_item.fixed_recovery,
                last_item.rolled_base,
                last_item.restored,
                last_item.hp_after,
            ] {
                hash.update(value.to_be_bytes());
            }
        }
        assert_eq!(
            format!("{:x}", hash.finalize()),
            "f11cc2762ad4866be61ea0ffc121bb6a2ee0632def4032091ef1c99af821b721"
        );
    }
}
