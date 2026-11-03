//! CPU-only course qualification against the exact bound S32 graph. Mirrors
//! ModernPetRace.qualify/enter/find: N,E,S,W, explicit sectors, ordered entry
//! from outside, and a closing edge into the next lap. Not race progress/ACK.
use super::{
    race_protocol::Gate,
    world_map::{WorldCell, WorldMap, DIRECTIONS},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeSet, VecDeque},
    sync::atomic::{AtomicBool, Ordering},
};

struct Work<'a> {
    left: u64,
    cancel: &'a AtomicBool,
}
impl Work<'_> {
    fn check(&self) -> Result<()> {
        ensure!(
            !self.cancel.load(Ordering::Acquire),
            "race geometry cancelled"
        );
        Ok(())
    }
    fn use_one(&mut self) -> Result<()> {
        self.left = self
            .left
            .checked_sub(1)
            .context("race geometry work budget exhausted")?;
        if self.left & 1023 == 0 {
            self.check()?;
        }
        Ok(())
    }
}
pub(crate) fn qualify(
    map: &WorldMap,
    gates: &[Gate],
    starts: &[WorldCell],
    initial: bool,
    cancel: &AtomicBool,
) -> Result<()> {
    qualify_checked(
        map,
        gates,
        starts,
        initial,
        &mut Work {
            left: 32_000_000,
            cancel,
        },
    )
}
fn qualify_checked(
    map: &WorldMap,
    gates: &[Gate],
    starts: &[WorldCell],
    initial: bool,
    work: &mut Work<'_>,
) -> Result<()> {
    work.check()?;
    ensure!(
        (2..=64).contains(&gates.len()) && (2..=10).contains(&starts.len()),
        "race geometry count outside bound"
    );
    ensure!(
        gates
            .iter()
            .enumerate()
            .all(|(i, g)| g.min_x <= g.max_x && g.min_y <= g.max_y && !gates[..i].contains(g)),
        "invalid race gates"
    );
    ensure!(
        starts.iter().all(|s| map.standing(*s)),
        "race runner outside shared terrain"
    );
    if initial {
        map.validate_spawns(starts, work.cancel)?;
    }
    let mut found = vec![false; gates.len()];
    // Do not iterate a remotely supplied rectangle: its coordinates may span
    // all of i32. At most 64 qualified sectors have only 4096 cells each.
    for binding in map.bindings() {
        let (ox, oy) = binding.key.world_origin();
        for y in 0..64 {
            for x in 0..64 {
                work.use_one()?;
                let cell = WorldCell {
                    x: ox.checked_add(x).context("race x overflow")?,
                    y: oy.checked_add(y).context("race y overflow")?,
                };
                if !map.standing(cell) {
                    continue;
                }
                let mut matches = 0;
                for (i, gate) in gates.iter().enumerate() {
                    work.use_one()?;
                    if gate.contains(cell) {
                        found[i] = true;
                        matches += 1;
                    }
                }
                ensure!(matches <= 1, "race checkpoints overlap on shared terrain");
            }
        }
    }
    ensure!(
        found.iter().all(|v| *v),
        "race checkpoint has no shared standable cell"
    );
    for start in starts {
        enter(map, *start, &gates[0], work)?;
    }
    let mut current = starts[0];
    for gate in gates {
        current = enter(map, current, gate, work)?;
    }
    enter(map, current, &gates[0], work)?;
    work.check()
}
fn enter(map: &WorldMap, from: WorldCell, gate: &Gate, work: &mut Work<'_>) -> Result<WorldCell> {
    let outside = if gate.contains(from) {
        find(map, from, |c| !gate.contains(c), work)?
    } else {
        from
    };
    find(map, outside, |c| gate.contains(c), work)
}
fn find(
    map: &WorldMap,
    from: WorldCell,
    target: impl Fn(WorldCell) -> bool,
    work: &mut Work<'_>,
) -> Result<WorldCell> {
    let mut queue = VecDeque::from([from]);
    let mut seen = BTreeSet::from([from]);
    while let Some(current) = queue.pop_front() {
        work.use_one()?;
        if target(current) {
            return Ok(current);
        }
        for heading in [0, 2, 4, 6] {
            work.use_one()?;
            let (dx, dy) = DIRECTIONS[heading];
            let (Some(x), Some(y)) = (current.x.checked_add(dx), current.y.checked_add(dy)) else {
                continue;
            };
            let next = WorldCell { x, y };
            if map.can_step(current, next) && seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    anyhow::bail!("race checkpoint is not reachable through shared directed edges")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{resources::SharedMapSectorKey, world_map::SectorInput};
    fn map() -> WorldMap {
        WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: SharedMapSectorKey {
                    map_id: 504,
                    sector_x: 0x8000,
                    sector_y: 0x8000,
                },
                bytes: vec![0; 49170],
            }],
            &AtomicBool::new(false),
        )
        .unwrap()
    }
    fn starts() -> [WorldCell; 2] {
        [
            WorldCell { x: 32770, y: 32770 },
            WorldCell { x: 32771, y: 32770 },
        ]
    }
    fn gates() -> [Gate; 2] {
        [
            Gate {
                min_x: 32773,
                max_x: 32774,
                min_y: 32770,
                max_y: 32772,
            },
            Gate {
                min_x: 32778,
                max_x: 32779,
                min_y: 32770,
                max_y: 32772,
            },
        ]
    }
    #[test]
    fn ordered_gates_close_next_lap_and_initial_spawn_overlap_is_not_accepted() {
        let map = map();
        let c = AtomicBool::new(false);
        qualify(&map, &gates(), &starts(), true, &c).unwrap();
        let overlap = [starts()[0]; 2];
        assert!(qualify(&map, &gates(), &overlap, true, &c).is_err());
        // Finished/retired runners may legitimately overlap; never turn that
        // into a new spawn proof. The course itself still has to close.
        qualify(&map, &gates(), &overlap, false, &c).unwrap();
    }
    #[test]
    fn overlap_empty_huge_rectangles_cancel_and_work_exhaustion_fail_closed() {
        let map = map();
        let c = AtomicBool::new(false);
        let mut g = gates();
        g[1].min_x = g[0].min_x;
        assert!(qualify(&map, &g, &starts(), true, &c).is_err());
        g = gates();
        g[0].min_x = i32::MIN;
        g[0].max_x = i32::MAX;
        g[0].min_y = i32::MIN;
        g[0].max_y = i32::MAX;
        assert!(qualify(&map, &g, &starts(), true, &c).is_err());
        g = gates();
        g[0].min_x = 1;
        g[0].max_x = 2;
        assert!(qualify(&map, &g, &starts(), true, &c).is_err());
        assert!(qualify(&map, &gates(), &starts(), true, &AtomicBool::new(true)).is_err());
        assert!(qualify_checked(
            &map,
            &gates(),
            &starts(),
            true,
            &mut Work {
                left: 1,
                cancel: &c
            }
        )
        .is_err());
    }
    #[test]
    fn checkpoint_cells_can_stand_but_must_be_reachable_and_reentered_from_outside() {
        let mut bytes = vec![0; 49170];
        for i in 0..4096 {
            bytes[32770 + i * 4] = 1;
            bytes[32770 + i * 4 + 2] = 1;
        }
        // Two disconnected two-cell islands, with all four cells standable.
        bytes[32770 + (2 * 64 + 2) * 4 + 2] = 0;
        bytes[32770 + (10 * 64 + 10) * 4 + 2] = 0;
        let map = WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: SharedMapSectorKey {
                    map_id: 504,
                    sector_x: 0x8000,
                    sector_y: 0x8000,
                },
                bytes,
            }],
            &AtomicBool::new(false),
        )
        .unwrap();
        let point = |x, y| Gate {
            min_x: x,
            max_x: x,
            min_y: y,
            max_y: y,
        };
        let a = point(32771, 32770);
        let b = point(32770, 32770);
        let remote = point(32779, 32778);
        assert!(map.standing(WorldCell { x: 32779, y: 32778 }));
        let c = AtomicBool::new(false);
        qualify(&map, &[a, b], &starts(), true, &c).unwrap();
        assert!(qualify(&map, &[a, remote], &starts(), true, &c).is_err());
    }
}
