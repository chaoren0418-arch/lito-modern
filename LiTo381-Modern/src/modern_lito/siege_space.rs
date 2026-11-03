//! Shared-S32 siege qualification. An ornamental door is not a cutset.
//! No LT data, client teleport destination, or optimistic gate opening.
use super::{
    siege_protocol::{Data, Scope},
    world_map::{WorldCell, WorldMap, DIRECTIONS},
    world_path,
    world_protocol::Frame,
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeSet, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub fn blockers(d: &Data) -> BTreeSet<WorldCell> {
    d.objectives
        .iter()
        .map(|t| t.cell)
        .chain(d.gate.iter().copied().filter(|_| !d.open()))
        .collect()
}

pub struct Space {
    scope: Scope,
    logical_map_id: u32,
    descriptor: Option<super::world_protocol::MatchDescriptor>,
    policy: Data,
    map: Arc<WorldMap>,
    regions: Regions,
}
/// Geometry-only input shared by both wire models. Neither caller constructs
/// a fake World/Combat frame or translates authoritative revisions.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Geometry {
    pub towers: Vec<WorldCell>,
    pub gate: Vec<WorldCell>,
    pub attack: Vec<WorldCell>,
    pub defense: Vec<WorldCell>,
    pub portal_outside: WorldCell,
    pub portal_inside: WorldCell,
}
impl Geometry {
    fn legacy(d: &Data) -> Self {
        Self {
            towers: d.objectives.iter().map(|t| t.cell).collect(),
            gate: d.gate.clone(),
            attack: d.attack_camp.clone(),
            defense: d.defense_camp.clone(),
            portal_outside: d.portal_outside,
            portal_inside: d.portal_inside,
        }
    }
}
pub(crate) struct Regions {
    outside: BTreeSet<WorldCell>,
    inside: BTreeSet<WorldCell>,
    opened: BTreeSet<WorldCell>,
}
impl Space {
    /// CPU-only, bounded/cancellable. The caller must run this on its owned
    /// resource worker before LOADED; the network/UI thread cannot flood-fill.
    pub fn qualify(map: Arc<WorldMap>, world: &Frame, cancel: &AtomicBool) -> Result<Self> {
        let key = world
            .actors
            .iter()
            .find_map(|a| a.identity().character())
            .context("siege human owner absent")?;
        world.validate(key)?;
        let d = world.siege.as_ref().context("siege policy absent")?;
        ensure!(
            world.manifest.digest == map.canonical_digest()
                && world.manifest.map_id == map.map_id(),
            "siege shared binding mismatch"
        );
        Self::qualify_checked(map, world, d, || {
            ensure!(
                !cancel.load(Ordering::Acquire),
                "siege qualification cancelled"
            );
            Ok(())
        })
    }
    fn qualify_checked(
        map: Arc<WorldMap>,
        world: &Frame,
        d: &Data,
        check: impl FnMut() -> Result<()>,
    ) -> Result<Self> {
        let regions = Regions::qualify_checked(&map, &Geometry::legacy(d), check)?;
        Ok(Self {
            scope: Scope::of(world),
            logical_map_id: world.room.summary.settings.map_id,
            descriptor: world.descriptor,
            policy: d.clone(),
            map,
            regions,
        })
    }
    pub fn matches(&self, w: &Frame) -> bool {
        Scope::of(w) == self.scope
            && w.room.summary.settings.mode == 3
            && w.room.summary.settings.map_id == self.logical_map_id
            && w.descriptor == self.descriptor
            && w.manifest.map_id == self.map.map_id()
            && w.manifest.digest == self.map.canonical_digest()
            && w.siege.as_ref().is_some_and(|d| self.policy.same_policy(d))
    }
    pub fn can_step(&self, w: &Frame, from: WorldCell, to: WorldCell) -> bool {
        self.matches(w)
            && w.siege
                .as_ref()
                .is_some_and(|d| world_path::step_open(&self.map, from, to, &blockers(d)))
    }
    pub fn teleport_allowed(&self, w: &Frame, from: WorldCell, to: WorldCell) -> bool {
        self.matches(w)
            && w.siege
                .as_ref()
                .is_some_and(|d| self.regions.allows(from, to, d.open()))
    }
}
impl Regions {
    pub(crate) fn qualify(
        map: &WorldMap,
        geometry: &Geometry,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::qualify_checked(map, geometry, || {
            ensure!(
                !cancel.load(Ordering::Acquire),
                "siege qualification cancelled"
            );
            Ok(())
        })
    }
    fn qualify_checked(
        map: &WorldMap,
        d: &Geometry,
        mut check: impl FnMut() -> Result<()>,
    ) -> Result<Self> {
        check()?;
        ensure!(
            d.towers.len() == 3
                && !d.gate.is_empty()
                && d.gate.len() <= 64
                && !d.attack.is_empty()
                && d.attack.len() <= 50
                && !d.defense.is_empty()
                && d.defense.len() <= 50,
            "siege geometry count outside bound"
        );
        ensure!(
            d.towers
                .iter()
                .chain(&d.gate)
                .chain(&d.attack)
                .chain(&d.defense)
                .chain([&d.portal_outside, &d.portal_inside])
                .all(|p| map.standing(*p)),
            "siege geometry is not shared standable terrain"
        );
        let towers: BTreeSet<_> = d.towers.iter().copied().collect();
        let closed: BTreeSet<_> = towers
            .iter()
            .copied()
            .chain(d.gate.iter().copied())
            .collect();
        let outside = reachable(map, d.attack[0], &closed, &mut check)?;
        let inside = reachable(map, d.defense[0], &closed, &mut check)?;
        ensure!(
            d.attack.iter().all(|p| outside.contains(p))
                && d.defense.iter().all(|p| inside.contains(p))
                && !outside.contains(&d.defense[0])
                && outside.contains(&d.portal_outside)
                && inside.contains(&d.portal_inside),
            "siege gate/camps/portals do not separate roles"
        );
        for (i, t) in d.towers.iter().enumerate() {
            ensure!(
                if i < 2 {
                    can_hit(map, &outside, *t, &closed)
                } else {
                    can_hit(map, &inside, *t, &closed) && !can_hit(map, &outside, *t, &closed)
                },
                "siege tower reachable from wrong side or unreachable"
            );
        }
        let opened = reachable(map, d.attack[0], &towers, &mut check)?;
        ensure!(
            inside.is_subset(&opened)
                && outside.is_subset(&opened)
                && d.gate.iter().all(|p| opened.contains(p)),
            "siege opened gate does not connect both sides"
        );
        check()?;
        Ok(Self {
            outside,
            inside,
            opened,
        })
    }
    pub(crate) fn allows(&self, from: WorldCell, to: WorldCell, open: bool) -> bool {
        if open {
            self.opened.contains(&from) && self.opened.contains(&to)
        } else {
            self.outside.contains(&from) && self.outside.contains(&to)
                || self.inside.contains(&from) && self.inside.contains(&to)
        }
    }
}
fn reachable(
    map: &WorldMap,
    start: WorldCell,
    blocked: &BTreeSet<WorldCell>,
    check: &mut impl FnMut() -> Result<()>,
) -> Result<BTreeSet<WorldCell>> {
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(from) = queue.pop_front() {
        check()?;
        for (dx, dy) in DIRECTIONS {
            let (Some(x), Some(y)) = (from.x.checked_add(dx), from.y.checked_add(dy)) else {
                continue;
            };
            let next = WorldCell { x, y };
            if world_path::step_open(map, from, next, blocked) && seen.insert(next) {
                queue.push_back(next);
            }
        }
        ensure!(
            seen.len() <= 65_536,
            "siege qualification exceeds shared-sector budget"
        );
    }
    Ok(seen)
}
fn can_hit(
    map: &WorldMap,
    region: &BTreeSet<WorldCell>,
    target: WorldCell,
    closed: &BTreeSet<WorldCell>,
) -> bool {
    let mut rays = closed.clone();
    rays.remove(&target);
    DIRECTIONS.into_iter().any(|(dx, dy)| {
        let (Some(x), Some(y)) = (target.x.checked_add(dx), target.y.checked_add(dy)) else {
            return false;
        };
        let from = WorldCell { x, y };
        region.contains(&from)
            && !closed.contains(&from)
            && world_path::step_open(map, from, target, &rays)
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::modern_lito::{
        resources::SharedMapSectorKey, siege_protocol::tests::sample, world_map::SectorInput,
    };
    pub(crate) fn map(world: &Frame) -> Arc<WorldMap> {
        let s = &world.manifest.sectors[0];
        Arc::new(
            WorldMap::from_sector_bytes(
                vec![SectorInput {
                    key: SharedMapSectorKey {
                        map_id: world.manifest.map_id,
                        sector_x: s.x,
                        sector_y: s.y,
                    },
                    bytes: vec![0; 49170],
                }],
                &AtomicBool::new(false),
            )
            .unwrap(),
        )
    }
    #[test]
    fn real_java_policy_matches_shared_terrain_and_gate_only_opens_after_both_towers() {
        let w = sample("active").frame.unwrap().world;
        let map = map(&w);
        let d = w.siege.as_ref().unwrap();
        let space = Space::qualify(map, &w, &AtomicBool::new(false)).unwrap();
        let before = d.portal_outside;
        let gate = WorldCell {
            x: before.x + 1,
            y: before.y,
        };
        let diagonal = WorldCell {
            x: gate.x,
            y: gate.y + 1,
        };
        for name in ["active", "left"] {
            let f = sample(name).frame.unwrap().world;
            assert!(!space.can_step(&f, before, gate));
            assert!(!space.can_step(&f, before, diagonal));
            assert!(!space.teleport_allowed(&f, d.attack_camp[0], d.defense_camp[0]));
        }
        let opened = sample("right").frame.unwrap().world;
        assert!(space.can_step(&opened, before, gate));
        assert!(space.can_step(&opened, before, diagonal));
        assert!(space.teleport_allowed(&opened, d.attack_camp[0], d.defense_camp[0]));
        for t in &d.objectives {
            assert!(!space.teleport_allowed(&opened, d.attack_camp[0], t.cell));
            assert!(blockers(opened.siege.as_ref().unwrap()).contains(&t.cell));
        }
        let mut foreign = opened.clone();
        foreign.match_id += 1;
        assert!(!space.matches(&foreign));
        assert!(!space.can_step(&foreign, before, gate));
        let mut altered = opened;
        altered.siege.as_mut().unwrap().attack_camp.swap(1, 2);
        assert!(!space.matches(&altered));
    }
    #[test]
    fn ornamental_gate_each_end_gap_wrong_roles_and_cancel_are_rejected() {
        let w = sample("active").frame.unwrap().world;
        let map = map(&w);
        for index in [0, 1, 10, 31, 62, 63] {
            let mut broken = w.clone();
            broken.siege.as_mut().unwrap().gate.remove(index);
            assert!(
                Space::qualify(map.clone(), &broken, &AtomicBool::new(false)).is_err(),
                "gap {index}"
            );
        }
        let mut door = w.clone();
        door.siege.as_mut().unwrap().gate.truncate(1);
        assert!(Space::qualify(map.clone(), &door, &AtomicBool::new(false)).is_err());
        assert!(Space::qualify(map.clone(), &w, &AtomicBool::new(true)).is_err());
        let mut calls = 0;
        assert!(
            Space::qualify_checked(map, &w, w.siege.as_ref().unwrap(), || {
                calls += 1;
                ensure!(calls < 20, "cancelled mid flood");
                Ok(())
            })
            .is_err()
        );
        assert_eq!(calls, 20);
    }
}
