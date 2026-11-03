//! Bounded, deterministic client-side planning over an immutable shared map.
//!
//! A plan is only intent: it does not move an actor, reserve cells, send a packet,
//! or prove current server occupancy. The caller must retain the selection/match
//! context, discard obsolete plans, and send each step from the last server-ACKed
//! cell/sequence. Replan when a server reply changes that state or occupancy.
//!
//! Equal-cost eight-direction BFS produces a shortest *step count*, not shortest
//! screen distance. Ties follow WorldMap's N, NE, E, SE, S, SW, W, NW order.
//! Static edges come exclusively from WorldMap. Occupied intermediate cells also
//! close their diagonal route: this client policy is deliberately more conservative
//! than the current server's destination-only actor occupancy check.
use super::world_map::{WorldCell, WorldMap, DIRECTIONS};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

pub const MAX_EXPANDED: usize = 65_536;
pub const MAX_STEPS: usize = 4_096;
pub const MAX_OCCUPIED: usize = 65_536;
// The shared WorldMap permits at most 16 sectors of 4096 cells. Independently
// cap discovery as well, so a future larger map cannot silently widen allocations.
const MAX_DISCOVERED: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathLimits {
    pub max_expanded: usize,
    /// Zero is useful for an already-at-target query; a nonempty path then
    /// reports PathTooLong, never a truncated success.
    pub max_steps: usize,
}
impl Default for PathLimits {
    fn default() -> Self {
        Self {
            max_expanded: MAX_EXPANDED,
            max_steps: MAX_STEPS,
        }
    }
}
impl PathLimits {
    fn validate(self) -> Result<(), PathError> {
        if self.max_expanded == 0 || self.max_expanded > MAX_EXPANDED || self.max_steps > MAX_STEPS
        {
            Err(PathError::InvalidLimits)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathError {
    InvalidLimits,
    OccupancyBudgetExceeded,
    StartOutside,
    GoalOutside,
    StartBlocked,
    GoalBlocked,
    StartOccupied,
    GoalOccupied,
    /// Every reachable cell was searched, with no valid path to the goal.
    NoPath,
    /// Search stopped at its work/discovery cap; this does not prove NoPath.
    SearchBudgetExceeded,
    /// BFS reached the goal, but its shortest path exceeds the requested cap.
    PathTooLong {
        required_steps: usize,
        limit: usize,
    },
    Cancelled,
    InternalInvariant,
}
impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits => f.write_str("invalid world path limits"),
            Self::OccupancyBudgetExceeded => f.write_str("world path occupancy budget exceeded"),
            Self::StartOutside => f.write_str("path start is outside the loaded map"),
            Self::GoalOutside => f.write_str("path goal is outside the loaded map"),
            Self::StartBlocked => f.write_str("path start is not standable"),
            Self::GoalBlocked => f.write_str("path goal is not standable"),
            Self::StartOccupied => f.write_str(
                "path occupancy still includes the selected actor or another actor at start",
            ),
            Self::GoalOccupied => f.write_str("path goal is occupied"),
            Self::NoPath => f.write_str("no path through the loaded map and occupancy snapshot"),
            Self::SearchBudgetExceeded => f.write_str("world path search budget exhausted"),
            Self::PathTooLong {
                required_steps,
                limit,
            } => write!(
                f,
                "shortest world path needs {required_steps} steps; limit is {limit}"
            ),
            Self::Cancelled => f.write_str("world path planning cancelled"),
            Self::InternalInvariant => f.write_str("world path predecessor invariant failed"),
        }
    }
}
impl std::error::Error for PathError {}

/// Immutable plan tagged with the exact loaded geometry. Cells includes both
/// start and goal; headings has exactly cells.len()-1 elements. No partial plan
/// is returned on cancellation, budget exhaustion, or excessive path length.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedPath {
    map_id: u32,
    manifest_digest: [u8; 32],
    cells: Vec<WorldCell>,
    headings: Vec<u8>,
    expanded: usize,
}
impl PlannedPath {
    pub fn map_id(&self) -> u32 {
        self.map_id
    }
    pub fn manifest_digest(&self) -> [u8; 32] {
        self.manifest_digest
    }
    pub fn cells(&self) -> &[WorldCell] {
        &self.cells
    }
    pub fn headings(&self) -> &[u8] {
        &self.headings
    }
    pub fn steps(&self) -> usize {
        self.headings.len()
    }
    pub fn expanded(&self) -> usize {
        self.expanded
    }
}

#[derive(Clone, Copy)]
struct Node {
    parent: Option<WorldCell>,
    heading: u8,
    depth: usize,
}

/// `occupied` must omit the selected actor. Other actors at start/goal are
/// rejected explicitly; unrelated out-of-map occupied cells are harmless.
/// The borrowed map/occupancy cannot be mutated during this call. Cancellation
/// is sampled before validation and throughout search/reconstruction; it is not
/// a time deadline or an atomic promise about cancellation after this returns.
pub fn plan(
    map: &WorldMap,
    start: WorldCell,
    goal: WorldCell,
    occupied: &BTreeSet<WorldCell>,
    limits: PathLimits,
    cancel: &AtomicBool,
) -> Result<PlannedPath, PathError> {
    plan_checked(map, start, goal, occupied, limits, || check_cancel(cancel))
}
fn check_cancel(cancel: &AtomicBool) -> Result<(), PathError> {
    if cancel.load(Ordering::Acquire) {
        Err(PathError::Cancelled)
    } else {
        Ok(())
    }
}

// The private checker keeps mid-search cancellation tests deterministic without
// threads/sleeps or a production API that accepts arbitrary navigation callbacks.
fn plan_checked(
    map: &WorldMap,
    start: WorldCell,
    goal: WorldCell,
    occupied: &BTreeSet<WorldCell>,
    limits: PathLimits,
    mut check: impl FnMut() -> Result<(), PathError>,
) -> Result<PlannedPath, PathError> {
    check()?;
    limits.validate()?;
    if occupied.len() > MAX_OCCUPIED {
        return Err(PathError::OccupancyBudgetExceeded);
    }
    if !map.contains(start) {
        return Err(PathError::StartOutside);
    }
    if !map.contains(goal) {
        return Err(PathError::GoalOutside);
    }
    if !map.standing(start) {
        return Err(PathError::StartBlocked);
    }
    if !map.standing(goal) {
        return Err(PathError::GoalBlocked);
    }
    if occupied.contains(&start) {
        return Err(PathError::StartOccupied);
    }
    if occupied.contains(&goal) {
        return Err(PathError::GoalOccupied);
    }
    if start == goal {
        check()?;
        return Ok(PlannedPath {
            map_id: map.map_id(),
            manifest_digest: map.canonical_digest(),
            cells: vec![start],
            headings: vec![],
            expanded: 0,
        });
    }
    let mut nodes = BTreeMap::from([(
        start,
        Node {
            parent: None,
            heading: 0,
            depth: 0,
        },
    )]);
    let mut queue = VecDeque::from([start]);
    let mut expanded = 0;
    while let Some(from) = queue.pop_front() {
        check()?;
        if expanded == limits.max_expanded {
            return Err(PathError::SearchBudgetExceeded);
        }
        expanded += 1;
        let depth = nodes
            .get(&from)
            .ok_or(PathError::InternalInvariant)?
            .depth
            .checked_add(1)
            .ok_or(PathError::InternalInvariant)?;
        for (heading, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
            check()?;
            let (Some(x), Some(y)) = (from.x.checked_add(dx), from.y.checked_add(dy)) else {
                continue;
            };
            let next = WorldCell { x, y };
            if nodes.contains_key(&next) || !step_open(map, from, next, occupied) {
                continue;
            }
            if nodes.len() == MAX_DISCOVERED {
                return Err(PathError::SearchBudgetExceeded);
            }
            nodes.insert(
                next,
                Node {
                    parent: Some(from),
                    heading: heading as u8,
                    depth,
                },
            );
            if next == goal {
                check()?;
                if depth > limits.max_steps {
                    return Err(PathError::PathTooLong {
                        required_steps: depth,
                        limit: limits.max_steps,
                    });
                }
                let mut cells = Vec::with_capacity(depth + 1);
                let mut headings = Vec::with_capacity(depth);
                let mut cursor = goal;
                cells.push(cursor);
                for remaining in (1..=depth).rev() {
                    check()?;
                    let node = nodes.get(&cursor).ok_or(PathError::InternalInvariant)?;
                    if node.depth != remaining {
                        return Err(PathError::InternalInvariant);
                    }
                    headings.push(node.heading);
                    cursor = node.parent.ok_or(PathError::InternalInvariant)?;
                    cells.push(cursor);
                }
                if cursor != start {
                    return Err(PathError::InternalInvariant);
                }
                cells.reverse();
                headings.reverse();
                check()?;
                return Ok(PlannedPath {
                    map_id: map.map_id(),
                    manifest_digest: map.canonical_digest(),
                    cells,
                    headings,
                    expanded,
                });
            }
            queue.push_back(next);
        }
    }
    check()?;
    Err(PathError::NoPath)
}

/// Occupancy-aware edge policy, also useful when validating the next planned
/// step against a newer acknowledged actor snapshot. This does not reserve it.
pub fn step_open(
    map: &WorldMap,
    from: WorldCell,
    to: WorldCell,
    occupied: &BTreeSet<WorldCell>,
) -> bool {
    if !map.standing(from)
        || !map.standing(to)
        || occupied.contains(&from)
        || occupied.contains(&to)
        || !map.can_step(from, to)
    {
        return false;
    }
    if from.x == to.x || from.y == to.y {
        return true;
    }
    let via_x = WorldCell { x: to.x, y: from.y };
    let via_y = WorldCell { x: from.x, y: to.y };
    [via_x, via_y]
        .into_iter()
        .any(|via| !occupied.contains(&via) && map.can_step(from, via) && map.can_step(via, to))
}

/// Recheck a planned heading against the latest acknowledged cell/occupancy.
/// Invalid headings and coordinate overflow are closed, and the diagonal policy
/// is exactly the one used by `plan`. A true result is still not a reservation.
pub fn can_step(
    map: &WorldMap,
    from: WorldCell,
    heading: u8,
    occupied: &BTreeSet<WorldCell>,
) -> bool {
    let Some(&(dx, dy)) = DIRECTIONS.get(usize::from(heading)) else {
        return false;
    };
    let (Some(x), Some(y)) = (from.x.checked_add(dx), from.y.checked_add(dy)) else {
        return false;
    };
    step_open(map, from, WorldCell { x, y }, occupied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        resources::SharedMapSectorKey,
        world_map::{SectorBinding, SectorInput},
    };
    const LAYER3: usize = 32770;
    fn cell(x: i32, y: i32) -> WorldCell {
        WorldCell {
            x: 32768 + x,
            y: 32768 + y,
        }
    }
    fn key(x: u16) -> SharedMapSectorKey {
        SharedMapSectorKey {
            map_id: 514,
            sector_x: x,
            sector_y: 0x8000,
        }
    }
    fn bytes(open: bool) -> Vec<u8> {
        let mut bytes = vec![0; 49170];
        if !open {
            for i in 0..4096 {
                bytes[LAYER3 + i * 4] = 1;
                bytes[LAYER3 + i * 4 + 2] = 1;
            }
        }
        bytes
    }
    fn map(bytes: Vec<u8>) -> WorldMap {
        WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: key(0x8000),
                bytes,
            }],
            &AtomicBool::new(false),
        )
        .unwrap()
    }
    fn edge(bytes: &mut [u8], a: (usize, usize), b: (usize, usize)) {
        let dx = b.0 as isize - a.0 as isize;
        let dy = b.1 as isize - a.1 as isize;
        let (x, y, offset) = match (dx, dy) {
            (0, -1) => (a.0, a.1, 0),
            (1, 0) => (a.0, a.1, 2),
            (0, 1) => (b.0, b.1, 0),
            (-1, 0) => (b.0, b.1, 2),
            _ => panic!("fixture cardinal edge"),
        };
        bytes[LAYER3 + (y * 64 + x) * 4 + offset] = 0;
    }
    fn corridor() -> WorldMap {
        let mut b = bytes(false);
        for x in 10..15 {
            edge(&mut b, (x, 10), (x + 1, 10));
        }
        map(b)
    }
    fn run(
        map: &WorldMap,
        start: WorldCell,
        goal: WorldCell,
        occupied: &BTreeSet<WorldCell>,
    ) -> Result<PlannedPath, PathError> {
        plan(
            map,
            start,
            goal,
            occupied,
            PathLimits::default(),
            &AtomicBool::new(false),
        )
    }
    fn valid_path(
        map: &WorldMap,
        path: &PlannedPath,
        start: WorldCell,
        goal: WorldCell,
        occupied: &BTreeSet<WorldCell>,
    ) {
        assert_eq!(path.cells().first(), Some(&start));
        assert_eq!(path.cells().last(), Some(&goal));
        assert_eq!(path.cells().len(), path.steps() + 1);
        assert!(path.steps() <= MAX_STEPS && path.expanded() <= MAX_EXPANDED);
        assert_eq!(path.map_id(), map.map_id());
        assert_eq!(path.manifest_digest(), map.canonical_digest());
        for (pair, heading) in path.cells().windows(2).zip(path.headings()) {
            assert!(step_open(map, pair[0], pair[1], occupied));
            assert!(can_step(map, pair[0], *heading, occupied));
            assert!(map.can_step(pair[0], pair[1]));
            let (dx, dy) = DIRECTIONS[*heading as usize];
            assert_eq!(
                pair[1],
                WorldCell {
                    x: pair[0].x + dx,
                    y: pair[0].y + dy
                }
            );
        }
    }
    #[test]
    fn shortest_eight_direction_route_and_deterministic_equal_cost_detour() {
        let map = map(bytes(true));
        let occupied = BTreeSet::from([cell(11, 10)]);
        let route = run(&map, cell(10, 10), cell(12, 10), &occupied).unwrap();
        assert_eq!(route.cells(), &[cell(10, 10), cell(11, 9), cell(12, 10)]);
        assert_eq!(route.headings(), &[1, 3]);
        assert_eq!(
            route,
            run(&map, cell(10, 10), cell(12, 10), &occupied).unwrap()
        );
        valid_path(&map, &route, cell(10, 10), cell(12, 10), &occupied);
        let direct = run(&map, cell(10, 10), cell(15, 13), &BTreeSet::new()).unwrap();
        assert_eq!(direct.steps(), 5);
        valid_path(&map, &direct, cell(10, 10), cell(15, 13), &BTreeSet::new());
    }
    #[test]
    fn narrow_route_and_occupied_choke_point_are_not_partially_successful() {
        let map = corridor();
        let route = run(&map, cell(10, 10), cell(15, 10), &BTreeSet::new()).unwrap();
        assert_eq!(route.headings(), &[2; 5]);
        valid_path(&map, &route, cell(10, 10), cell(15, 10), &BTreeSet::new());
        assert_eq!(
            run(
                &map,
                cell(10, 10),
                cell(15, 10),
                &BTreeSet::from([cell(12, 10)])
            ),
            Err(PathError::NoPath)
        );
    }
    #[test]
    fn geometry_corner_needs_a_complete_route_and_occupied_corner_is_conservative() {
        let mut b = bytes(false);
        edge(&mut b, (10, 10), (11, 10));
        edge(&mut b, (11, 10), (11, 9));
        let open = map(b);
        let route = run(&open, cell(10, 10), cell(11, 9), &BTreeSet::new()).unwrap();
        assert_eq!(route.headings(), &[1]);
        assert_eq!(
            run(
                &open,
                cell(10, 10),
                cell(11, 9),
                &BTreeSet::from([cell(11, 10)])
            ),
            Err(PathError::NoPath)
        );
        let mut b = bytes(false);
        edge(&mut b, (10, 10), (11, 10));
        edge(&mut b, (11, 9), (12, 9));
        let closed = map(b);
        assert!(closed.standing(cell(11, 9)));
        assert!(!closed.can_step(cell(10, 10), cell(11, 9)));
        assert_eq!(
            run(&closed, cell(10, 10), cell(11, 9), &BTreeSet::new()),
            Err(PathError::NoPath)
        );
        let map = map(bytes(true));
        let occupied = BTreeSet::from([cell(11, 10), cell(10, 9)]);
        assert!(map.can_step(cell(10, 10), cell(11, 9)));
        assert!(!step_open(&map, cell(10, 10), cell(11, 9), &occupied));
        assert!(!can_step(&map, cell(10, 10), 1, &occupied));
        assert!(can_step(&map, cell(10, 10), 1, &BTreeSet::new()));
        for heading in [8, 255] {
            assert!(!can_step(&map, cell(10, 10), heading, &BTreeSet::new()));
        }
        assert!(!can_step(
            &map,
            WorldCell { x: i32::MAX, y: 0 },
            2,
            &BTreeSet::new()
        ));
        assert!(!can_step(
            &map,
            WorldCell { x: 0, y: i32::MIN },
            0,
            &BTreeSet::new()
        ));
    }
    #[test]
    fn endpoints_have_distinct_outside_blocked_and_occupied_errors() {
        let map = corridor();
        let none = BTreeSet::new();
        for (start, goal, error) in [
            (cell(-1, 10), cell(15, 10), PathError::StartOutside),
            (cell(10, 10), cell(64, 10), PathError::GoalOutside),
            (cell(0, 0), cell(15, 10), PathError::StartBlocked),
            (cell(10, 10), cell(0, 0), PathError::GoalBlocked),
            (
                WorldCell {
                    x: i32::MIN,
                    y: i32::MAX,
                },
                cell(15, 10),
                PathError::StartOutside,
            ),
        ] {
            assert_eq!(run(&map, start, goal, &none), Err(error));
        }
        assert_eq!(
            run(
                &map,
                cell(10, 10),
                cell(15, 10),
                &BTreeSet::from([cell(10, 10)])
            ),
            Err(PathError::StartOccupied)
        );
        assert_eq!(
            run(
                &map,
                cell(10, 10),
                cell(15, 10),
                &BTreeSet::from([cell(15, 10)])
            ),
            Err(PathError::GoalOccupied)
        );
        let route = run(
            &map,
            cell(10, 10),
            cell(15, 10),
            &BTreeSet::from([WorldCell {
                x: i32::MAX,
                y: i32::MIN,
            }]),
        )
        .unwrap();
        assert_eq!(route.steps(), 5);
    }
    #[test]
    fn same_valid_cell_has_zero_steps_and_never_bypasses_endpoint_checks() {
        let map = corridor();
        let limits = PathLimits {
            max_expanded: 1,
            max_steps: 0,
        };
        let path = plan(
            &map,
            cell(10, 10),
            cell(10, 10),
            &BTreeSet::new(),
            limits,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(path.cells(), &[cell(10, 10)]);
        assert!(path.headings().is_empty());
        assert_eq!(path.expanded(), 0);
        assert_eq!(
            plan(
                &map,
                cell(10, 10),
                cell(11, 10),
                &BTreeSet::new(),
                limits,
                &AtomicBool::new(false)
            ),
            Err(PathError::PathTooLong {
                required_steps: 1,
                limit: 0
            })
        );
        assert_eq!(
            run(
                &map,
                cell(10, 10),
                cell(10, 10),
                &BTreeSet::from([cell(10, 10)])
            ),
            Err(PathError::StartOccupied)
        );
    }
    #[test]
    fn search_budget_is_not_no_path_and_step_limit_is_not_a_truncated_path() {
        let map = corridor();
        let no = BTreeSet::new();
        let cancel = AtomicBool::new(false);
        assert_eq!(
            plan(
                &map,
                cell(10, 10),
                cell(15, 10),
                &no,
                PathLimits {
                    max_expanded: 1,
                    max_steps: MAX_STEPS
                },
                &cancel
            ),
            Err(PathError::SearchBudgetExceeded)
        );
        assert_eq!(
            plan(
                &map,
                cell(10, 10),
                cell(15, 10),
                &no,
                PathLimits {
                    max_expanded: 5,
                    max_steps: 4
                },
                &cancel
            ),
            Err(PathError::PathTooLong {
                required_steps: 5,
                limit: 4
            })
        );
        let path = plan(
            &map,
            cell(10, 10),
            cell(15, 10),
            &no,
            PathLimits {
                max_expanded: 5,
                max_steps: 5,
            },
            &cancel,
        )
        .unwrap();
        assert_eq!(path.expanded(), 5);
        for limits in [
            PathLimits {
                max_expanded: 0,
                max_steps: 1,
            },
            PathLimits {
                max_expanded: MAX_EXPANDED + 1,
                max_steps: 1,
            },
            PathLimits {
                max_expanded: 1,
                max_steps: MAX_STEPS + 1,
            },
        ] {
            assert_eq!(
                plan(&map, cell(10, 10), cell(10, 10), &no, limits, &cancel),
                Err(PathError::InvalidLimits)
            );
        }
    }
    #[test]
    fn occupancy_input_is_independently_bounded() {
        let map = corridor();
        let occupied = (0..=MAX_OCCUPIED)
            .map(|x| WorldCell { x: x as i32, y: -1 })
            .collect();
        assert_eq!(
            run(&map, cell(10, 10), cell(15, 10), &occupied),
            Err(PathError::OccupancyBudgetExceeded)
        );
    }
    #[test]
    fn cancellation_before_and_during_search_is_deterministic_and_returns_no_plan() {
        let map = map(bytes(true));
        let cancel = AtomicBool::new(true);
        assert_eq!(
            plan(
                &map,
                cell(0, 0),
                cell(63, 63),
                &BTreeSet::new(),
                PathLimits::default(),
                &cancel
            ),
            Err(PathError::Cancelled)
        );
        cancel.store(false, Ordering::Release);
        let mut checks = 0;
        let result = plan_checked(
            &map,
            cell(0, 0),
            cell(63, 63),
            &BTreeSet::new(),
            PathLimits::default(),
            || {
                checks += 1;
                if checks == 32 {
                    cancel.store(true, Ordering::Release);
                }
                check_cancel(&cancel)
            },
        );
        assert_eq!(result, Err(PathError::Cancelled));
        assert_eq!(checks, 32);
    }
    #[test]
    fn loaded_sector_edge_is_walkable_but_missing_sector_is_never_inferred() {
        let one = map(bytes(true));
        assert_eq!(
            run(&one, cell(63, 10), cell(65, 10), &BTreeSet::new()),
            Err(PathError::GoalOutside)
        );
        let two = WorldMap::from_sector_bytes(
            vec![
                SectorInput {
                    key: key(0x8000),
                    bytes: bytes(true),
                },
                SectorInput {
                    key: key(0x8001),
                    bytes: bytes(true),
                },
            ],
            &AtomicBool::new(false),
        )
        .unwrap();
        let path = run(&two, cell(63, 10), cell(65, 10), &BTreeSet::new()).unwrap();
        assert_eq!(path.steps(), 2);
        valid_path(&two, &path, cell(63, 10), cell(65, 10), &BTreeSet::new());
        let gap = WorldMap::from_sector_bytes(
            vec![
                SectorInput {
                    key: key(0x8000),
                    bytes: bytes(true),
                },
                SectorInput {
                    key: key(0x8002),
                    bytes: bytes(true),
                },
            ],
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(
            run(&gap, cell(63, 10), cell(128, 10), &BTreeSet::new()),
            Err(PathError::NoPath)
        );
    }
    #[test]
    fn portal_is_a_closed_cell_and_cannot_be_routed_through() {
        let mut b = bytes(false);
        for x in 10..15 {
            edge(&mut b, (x, 10), (x + 1, 10));
        }
        b.truncate(49166);
        b.extend(1u16.to_le_bytes());
        b.extend([0, 24, 10]);
        b.extend(4u16.to_le_bytes());
        b.extend(1i32.to_le_bytes());
        b.extend(0u16.to_le_bytes());
        let map = map(b);
        assert!(!map.standing(cell(12, 10)));
        assert_eq!(
            run(&map, cell(10, 10), cell(15, 10), &BTreeSet::new()),
            Err(PathError::NoPath)
        );
        assert_eq!(
            run(&map, cell(10, 10), cell(12, 10), &BTreeSet::new()),
            Err(PathError::GoalBlocked)
        );
    }
    #[test]
    fn hard_4096_step_limit_rejects_a_real_long_connected_snake() {
        // Four adjacent sectors, 17 long horizontal corridors. Vertical edges
        // exist only at alternating endpoints; legal diagonals shorten turns,
        // but cannot shortcut the walls between the rows.
        let mut sectors: Vec<_> = (0..4).map(|_| bytes(false)).collect();
        for y in 0..17 {
            for x in 0..255 {
                let sector = x / 64;
                let local = x % 64;
                sectors[sector][LAYER3 + (y * 64 + local) * 4 + 2] = 0;
            }
            if y < 16 {
                let x = if y % 2 == 0 { 255 } else { 0 };
                sectors[x / 64][LAYER3 + ((y + 1) * 64 + x % 64) * 4] = 0;
            }
        }
        let map = WorldMap::from_sector_bytes(
            sectors
                .into_iter()
                .enumerate()
                .map(|(i, bytes)| SectorInput {
                    key: key(0x8000 + i as u16),
                    bytes,
                })
                .collect(),
            &AtomicBool::new(false),
        )
        .unwrap();
        let result = run(&map, cell(0, 0), cell(255, 16), &BTreeSet::new());
        assert!(
            matches!(result,Err(PathError::PathTooLong{required_steps,limit:MAX_STEPS}) if required_steps>MAX_STEPS)
        );
    }
    fn digest(value: &str) -> [u8; 32] {
        let mut bytes = [0; 32];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = u8::from_str_radix(&value[2 * i..2 * i + 2], 16).unwrap();
        }
        bytes
    }
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; explicit shared514 read only, no output resources"]
    fn real_shared514_anchor_paths_use_every_authoritative_shared_step() {
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
        );
        let binding = SectorBinding {
            key: SharedMapSectorKey {
                map_id: 514,
                sector_x: 0x7fff,
                sector_y: 0x8000,
            },
            byte_length: 137784,
            sha256: digest("ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271"),
        };
        let map = WorldMap::load_shared(&root, &[binding], &AtomicBool::new(false)).unwrap();
        assert_eq!(
            map.canonical_digest(),
            digest("2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8")
        );
        let start = WorldCell { x: 32724, y: 32784 };
        assert_eq!(map.direction_mask(start), 60);
        for goal in [
            WorldCell { x: 32725, y: 32784 },
            WorldCell { x: 32725, y: 32785 },
            WorldCell { x: 32724, y: 32785 },
            WorldCell { x: 32723, y: 32785 },
        ] {
            let path = run(&map, start, goal, &BTreeSet::new()).unwrap();
            assert_eq!(path.steps(), 1);
            valid_path(&map, &path, start, goal, &BTreeSet::new());
            let back = run(&map, goal, start, &BTreeSet::new()).unwrap();
            valid_path(&map, &back, goal, start, &BTreeSet::new());
            assert_eq!(
                run(&map, start, goal, &BTreeSet::from([goal])),
                Err(PathError::GoalOccupied)
            );
        }
    }
}
