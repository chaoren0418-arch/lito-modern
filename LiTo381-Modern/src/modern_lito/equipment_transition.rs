//! Numeric contract for future server-confirmed in-match equipment changes.
//! This is not prediction or permission: legacy combat snapshots still forbid
//! maxima/AC/weapon changes until a scoped, versioned receipt is implemented.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vitals {
    pub hp: i64,
    pub maximum_hp: i64,
    pub mp: i64,
    pub maximum_mp: i64,
}
impl Vitals {
    pub fn valid(self) -> bool {
        self.maximum_hp > 0
            && self.maximum_mp >= 0
            && (0..=self.maximum_hp).contains(&self.hp)
            && (0..=self.maximum_mp).contains(&self.mp)
    }

    /// Retain absolute live values, clipping only excess above new maxima.
    /// Never fills HP/MP, restores a previous loss, revives, narrows, or reads SQL.
    pub fn retain(self, maximum_hp: i64, maximum_mp: i64) -> Option<Self> {
        if !self.valid() || maximum_hp < 1 || maximum_mp < 0 {
            return None;
        }
        Some(Self {
            hp: self.hp.min(maximum_hp),
            maximum_hp,
            mp: self.mp.min(maximum_mp),
            maximum_mp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_and_generated_wide_corpus_matches_java_contract() {
        use sha2::{Digest, Sha256};
        let mut rows = vec![
            [70, 100, 30, 50, 1000, 900],
            [70, 100, 30, 50, 10, 2],
            [0, 1, 0, 0, i64::MAX, i64::MAX],
            [i64::MAX, i64::MAX, i64::MAX, i64::MAX, 1, 0],
            [
                9007199254740993,
                i64::MAX,
                9007199254740993,
                i64::MAX,
                9007199254740992,
                i64::MAX,
            ],
            [-1, 100, 0, 0, 100, 0],
            [101, 100, 0, 0, 100, 0],
            [0, 0, 0, 0, 1, 0],
            [1, 100, -1, 10, 100, 10],
            [1, 100, 11, 10, 100, 10],
            [1, 100, 0, -1, 100, 10],
            [1, 100, 0, 0, 0, 0],
            [1, 100, 0, 0, -1, 0],
            [1, 100, 0, 0, 1, -1],
        ];
        let mut rng = 0x123456789abcdef0u64;
        for _ in 0..512 {
            let values: [i64; 4] = std::array::from_fn(|_| {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                (rng & i64::MAX as u64) as i64
            });
            rows.push([
                values[0],
                i64::MAX,
                values[1],
                i64::MAX,
                values[2].max(1),
                values[3],
            ]);
        }
        let mut digest = Sha256::new();
        for row in &rows {
            for value in row {
                digest.update(value.to_be_bytes());
            }
            let result = Vitals {
                hp: row[0],
                maximum_hp: row[1],
                mp: row[2],
                maximum_mp: row[3],
            }
            .retain(row[4], row[5]);
            digest.update([u8::from(result.is_some())]);
            if let Some(v) = result {
                for value in [v.hp, v.maximum_hp, v.mp, v.maximum_mp] {
                    digest.update(value.to_be_bytes());
                }
            }
        }
        let hash = format!("{:x}", digest.finalize());
        assert_eq!(rows.len(), 526);
        assert_eq!(
            hash,
            "ab1351a9d238c53914bfd9673220c812a94e21460b4a6cab6e3c3649d01eface"
        );
        println!("equipment-vitals-v1:{hash}:{}", rows.len());
    }
    #[test]
    fn increasing_maxima_never_heals_or_restores_a_prior_clipped_loss() {
        let before = Vitals {
            hp: 70,
            maximum_hp: 100,
            mp: 30,
            maximum_mp: 50,
        };
        let up = before.retain(1000, 900).unwrap();
        assert_eq!((up.hp, up.mp), (70, 30));
        let down = up.retain(10, 2).unwrap();
        assert_eq!((down.hp, down.mp), (10, 2));
        let again = down.retain(1000, 900).unwrap();
        assert_eq!((again.hp, again.mp), (10, 2));
        assert_eq!((before.hp, before.mp), (70, 30));
    }
    #[test]
    fn dead_and_zero_mana_stay_zero_across_wide_maximum_changes() {
        let dead = Vitals {
            hp: 0,
            maximum_hp: i64::MAX,
            mp: 0,
            maximum_mp: i64::MAX,
        };
        for (hp, mp) in [(1, 0), (i64::MAX, i64::MAX)] {
            let retained = dead.retain(hp, mp).unwrap();
            assert_eq!((retained.hp, retained.mp), (0, 0));
        }
        let alive = Vitals {
            hp: i64::MAX,
            maximum_hp: i64::MAX,
            mp: i64::MAX,
            maximum_mp: i64::MAX,
        };
        assert_eq!(alive.retain(i64::MAX, i64::MAX), Some(alive));
        assert_eq!(
            alive.retain(1, 0).unwrap(),
            Vitals {
                hp: 1,
                maximum_hp: 1,
                mp: 0,
                maximum_mp: 0
            }
        );
    }
    #[test]
    fn legacy_combat_wire_still_rejects_unreceipted_equipment_changes() {
        use crate::modern_lito::{combat_protocol as combat, item_protocol};
        let before = item_protocol::tests::initial();
        let mut heartbeat = before.clone();
        heartbeat.frame.as_mut().unwrap().world.revision += 1;
        heartbeat.validate().unwrap();
        assert!(combat::follows(Some(&before), &heartbeat));
        for field in 0..5 {
            let mut changed = heartbeat.clone();
            let vital = &mut changed.frame.as_mut().unwrap().vitals[0];
            match field {
                0 => vital.max_hp += 1,
                1 => vital.max_mp += 1,
                2 => vital.ac += 1,
                3 => vital.weapon_type = 1,
                _ => vital.attack_interval_nanos += 1,
            }
            changed.validate().unwrap();
            assert!(
                !combat::follows(Some(&before), &changed),
                "field {field} requires a scoped equipment receipt, not this numeric helper"
            );
        }
    }
    #[test]
    fn invalid_prior_or_candidate_is_rejected_instead_of_sanitized() {
        for (hp, max_hp, mp, max_mp, next_hp, next_mp) in [
            (-1, 100, 0, 0, 100, 0),
            (101, 100, 0, 0, 100, 0),
            (0, 0, 0, 0, 1, 0),
            (1, 100, -1, 10, 100, 10),
            (1, 100, 11, 10, 100, 10),
            (1, 100, 0, -1, 100, 10),
            (1, 100, 0, 0, 0, 0),
            (1, 100, 0, 0, -1, 0),
            (1, 100, 0, 0, 1, -1),
        ] {
            assert!(Vitals {
                hp,
                maximum_hp: max_hp,
                mp,
                maximum_mp: max_mp
            }
            .retain(next_hp, next_mp)
            .is_none());
        }
    }
}
