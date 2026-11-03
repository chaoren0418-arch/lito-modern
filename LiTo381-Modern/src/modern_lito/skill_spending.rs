//! Candidate accumulated human spell ledger for dynamic equipment epochs.
//! Qualified INT-reduced Healing can cost zero; a free cast still counts.
//! Not advertised on the wire; source/world/life/skill and event revision must
//! be bound by the future negotiated envelope. Existing fixed-cost validation
//! remains unchanged. No MP prediction or skill permission is granted here.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Totals {
    pub casts: i64,
    pub spent: i64,
}
impl Totals {
    fn valid(self) -> bool {
        self.casts >= 0
            && self.spent >= 0
            && (self.casts != 0 || self.spent == 0)
            && (self.casts == 0
                || self.spent / self.casts < i64::from(i32::MAX)
                || self.spent / self.casts == i64::from(i32::MAX) && self.spent % self.casts == 0)
    }
    pub fn advance(self, cost: i32) -> Option<Self> {
        if !self.valid() || cost < 0 {
            return None;
        }
        Some(Self {
            casts: self.casts.checked_add(1)?,
            spent: self.spent.checked_add(i64::from(cost))?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ledger {
    pub equipment_revision: i64,
    pub profile_revision: i64,
    pub cost: i32,
    pub prior_casts: i64,
    pub prior_spent: i64,
    pub casts: i64,
    pub spent: i64,
    pub last_cast_revision: i64,
    pub last_cast_sequence: i64,
}
impl Ledger {
    pub fn valid(self) -> bool {
        let prior = Totals {
            casts: self.prior_casts,
            spent: self.prior_spent,
        };
        let total = Totals {
            casts: self.casts,
            spent: self.spent,
        };
        if !prior.valid()
            || !total.valid()
            || self.equipment_revision < 0
            || self.profile_revision < 1
            || self.cost < 0
            || self.casts < self.prior_casts
            || self.spent < self.prior_spent
            || self.last_cast_revision < 0
            || self.last_cast_sequence < self.casts
            || self.casts > self.last_cast_revision
            || self.prior_casts >= self.profile_revision
            || (self.casts == 0) != (self.last_cast_revision == 0)
            || self.casts == 0 && self.last_cast_sequence != 0
            || self.casts == self.prior_casts && self.last_cast_revision >= self.profile_revision
            || self.casts > self.prior_casts && self.last_cast_revision <= self.profile_revision
        {
            return false;
        }
        (self.casts - self.prior_casts)
            .checked_mul(i64::from(self.cost))
            .and_then(|v| v.checked_add(self.prior_spent))
            == Some(self.spent)
    }
    pub fn initial(equipment_revision: i64, profile_revision: i64, cost: i32) -> Option<Self> {
        let value = Self {
            equipment_revision,
            profile_revision,
            cost,
            prior_casts: 0,
            prior_spent: 0,
            casts: 0,
            spent: 0,
            last_cast_revision: 0,
            last_cast_sequence: 0,
        };
        value.valid().then_some(value)
    }
    pub fn cast(self, revision: i64, sequence: i64) -> Option<Self> {
        if !self.valid()
            || revision <= self.profile_revision
            || revision <= self.last_cast_revision
            || sequence <= self.last_cast_sequence
        {
            return None;
        }
        let total = Totals {
            casts: self.casts,
            spent: self.spent,
        }
        .advance(self.cost)?;
        let value = Self {
            casts: total.casts,
            spent: total.spent,
            last_cast_revision: revision,
            last_cast_sequence: sequence,
            ..self
        };
        value.valid().then_some(value)
    }
    pub fn reprice(self, equipment_revision: i64, revision: i64, cost: i32) -> Option<Self> {
        if !self.valid()
            || self.equipment_revision.checked_add(1)? != equipment_revision
            || revision <= self.profile_revision
            || revision <= self.last_cast_revision
        {
            return None;
        }
        let value = Self {
            equipment_revision,
            profile_revision: revision,
            cost,
            prior_casts: self.casts,
            prior_spent: self.spent,
            ..self
        };
        value.valid().then_some(value)
    }
}

pub fn follows(before: Ledger, after: Ledger) -> bool {
    before.valid()
        && after.valid()
        && (before == after
            || before.cast(after.last_cast_revision, after.last_cast_sequence) == Some(after)
            || before.reprice(after.equipment_revision, after.profile_revision, after.cost)
                == Some(after))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn cost_changes_preserve_spent_totals_and_historical_casts() {
        let first = Ledger::initial(9, 1, 70)
            .unwrap()
            .cast(2, 1)
            .unwrap()
            .cast(3, 3)
            .unwrap();
        let changed = first.reprice(10, 4, 59).unwrap();
        assert_eq!(
            (
                changed.casts,
                changed.spent,
                changed.last_cast_revision,
                changed.last_cast_sequence
            ),
            (2, 140, 3, 3)
        );
        let next = changed.cast(5, 8).unwrap();
        assert_eq!(next.spent, 199);
        assert_eq!(
            next.reprice(11, 6, 70).unwrap().cast(7, 9).unwrap().spent,
            269
        );
        assert!(follows(first, changed) && follows(changed, next) && follows(next, next));
        assert!(!follows(first, next));
    }

    #[test]
    fn repeated_empty_epochs_never_invent_spending() {
        let mut value = Ledger::initial(0, 1, 3).unwrap();
        for i in 1..=1000 {
            let next = value.reprice(i, i + 1, 3).unwrap();
            assert!(follows(value, next));
            value = next;
        }
        assert_eq!(
            (value.casts, value.spent, value.last_cast_sequence),
            (0, 0, 0)
        );
        assert_eq!(value.cast(1002, 1).unwrap().spent, 3);
    }

    #[test]
    fn invalid_and_rewritten_history_cannot_follow() {
        let a = Ledger::initial(0, 1, 3).unwrap().cast(2, 1).unwrap();
        assert!(a.reprice(2, 3, 1).is_none());
        assert!(a.reprice(1, 2, 1).is_none());
        assert!(a.reprice(1, 3, -1).is_none());
        assert!(a.cast(3, 1).is_none());
        assert!(a.cast(2, 2).is_none());
        let b = a.reprice(1, 3, 1).unwrap();
        let forged = Ledger {
            prior_spent: 4,
            spent: 4,
            ..b
        };
        assert!(forged.valid());
        assert!(!follows(a, forged) && !follows(b, forged));
        assert!(!follows(a, b.cast(4, 2).unwrap()));
        let broken = Ledger { spent: -1, ..a };
        assert!(!follows(broken, broken));
        assert!(broken.cast(4, 2).is_none());
    }

    #[test]
    fn wide_values_and_overflow_match_signed_i64_without_partial_updates() {
        let n = 9_007_199_254_740_993;
        let wide = Ledger {
            equipment_revision: 1,
            profile_revision: n + 2,
            cost: 1,
            prior_casts: n,
            prior_spent: n,
            casts: n,
            spent: n,
            last_cast_revision: n + 1,
            last_cast_sequence: n,
        };
        assert_eq!(wide.cast(n + 3, n + 1).unwrap().spent, n + 1);
        let half = i64::MAX / 2 + 1;
        let full = Ledger {
            equipment_revision: i64::MAX,
            profile_revision: half + 2,
            prior_casts: half,
            casts: half,
            last_cast_revision: half + 1,
            last_cast_sequence: half,
            prior_spent: i64::MAX,
            spent: i64::MAX,
            ..wide
        };
        assert!(full.valid());
        assert!(full.cast(half + 3, half + 1).is_none());
        assert!(full.reprice(0, half + 3, 1).is_none());
        assert_eq!(full.spent, i64::MAX);
        assert!(Totals {
            casts: -1,
            spent: 1
        }
        .advance(1)
        .is_none());
        assert!(!Totals {
            casts: 1,
            spent: i64::MAX
        }
        .valid());
        assert!(Totals { casts: 0, spent: 1 }.advance(1).is_none());
        assert!(Ledger::initial(-1, 1, 1).is_none());
        assert!(Ledger::initial(0, 0, 1).is_none());
        assert!(!Ledger {
            equipment_revision: 0,
            profile_revision: 1,
            cost: 2,
            prior_casts: 0,
            prior_spent: 0,
            casts: i64::MAX,
            spent: i64::MAX,
            last_cast_revision: i64::MAX,
            last_cast_sequence: i64::MAX
        }
        .valid());
    }

    #[test]
    fn lawful_zero_cost_healing_counts_without_spending_or_refunding_mp() {
        let free = Ledger::initial(0, 1, 0).unwrap().cast(2, 1).unwrap();
        assert_eq!((free.casts, free.spent), (1, 0));
        let paid = free.reprice(1, 3, 4).unwrap().cast(4, 2).unwrap();
        let free_again = paid.reprice(2, 5, 0).unwrap().cast(6, 3).unwrap();
        assert_eq!((free_again.casts, free_again.spent), (3, 4));
        assert!(follows(paid, paid.reprice(2, 5, 0).unwrap()));
    }

    #[test]
    fn generated_epoch_corpus_matches_java() {
        let mut digest = Sha256::new();
        let mut rng = 0x1234_5678_9abc_def0_u64;
        let mut state = Ledger::initial(0, 1, 70).unwrap();
        for i in 1..=512_i64 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let next = if i % 3 == 0 {
                state.reprice(state.equipment_revision + 1, i + 1, (rng % 71) as i32)
            } else {
                state.cast(i + 1, 2 * i)
            }
            .unwrap();
            assert!(follows(state, next));
            for value in [
                next.equipment_revision,
                next.profile_revision,
                i64::from(next.cost),
                next.prior_casts,
                next.prior_spent,
                next.casts,
                next.spent,
                next.last_cast_revision,
                next.last_cast_sequence,
            ] {
                digest.update(value.to_be_bytes());
            }
            state = next;
        }
        let hash = format!("{:x}", digest.finalize());
        assert_eq!(
            hash,
            "c29985c2af1e8427d846dea8a29b97fff80b6782d8d79fc19b5e65e0f355831a"
        );
        eprintln!("skill-spending-v1:{hash}:512");
    }
}
