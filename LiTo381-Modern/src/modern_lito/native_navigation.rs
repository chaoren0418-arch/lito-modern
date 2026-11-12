//! Formal LTmap directional edges; deliberately independent of Modern v1 masks.
use super::world_map::WorldMap;
use anyhow::{ensure, Context, Result};
use std::{cmp::Reverse, collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque}};
#[cfg(test)]
#[path = "native_collision_tests.rs"]
mod collision_tests;
const DIR: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];
#[derive(Debug)]
pub struct Navigation {
    pub map_id: u32,
    sectors: BTreeMap<(i32, i32), Box<[u8]>>,
    portal_sources: BTreeSet<(u16,u16)>,
}
impl Navigation {
    pub fn from_geometry(map: &WorldMap) -> Result<Self> {
        let mut sectors = BTreeMap::new();
        let mut portal_sources = BTreeSet::new();
        for b in map.bindings() {
            let s = map.sector(b.key).unwrap();
            ensure!(s.attributes.len() == 4096, "navigation sector size");
            let (x, y) = b.key.world_origin();
            for portal in &s.portals {
                // Native entrance tags share the source-only C73 request.
                // Team permissions and destinations remain server authority.
                if matches!(portal.native_kind, Some(100..=102)) {
                    let point=(u16::try_from(x+i32::from(portal.x)/2)?,u16::try_from(y+i32::from(portal.y))?);
                    ensure!(portal_sources.insert(point),"ambiguous native portal source");
                }
            }
            sectors.insert(
                (x.div_euclid(64), y.div_euclid(64)),
                s.attributes
                    .iter()
                    .map(|a| (a.attribute1 as u8 & 1) | ((a.attribute2 as u8 & 1) << 1))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
        }
        Ok(Self {
            map_id: map.map_id(),
            sectors,
            portal_sources,
        })
    }
    pub fn portal_sources(&self)->impl Iterator<Item=[u16;2]>+'_ {
        self.portal_sources.iter().map(|&(x,y)|[x,y])
    }
    pub fn has_portal_source(&self,point:[u16;2])->bool {
        self.portal_sources.contains(&(point[0],point[1]))
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.sectors
            .contains_key(&(x.div_euclid(64), y.div_euclid(64)))
    }
    fn edge(&self, x: i32, y: i32, bit: u8) -> bool {
        self.sectors
            .get(&(x.div_euclid(64), y.div_euclid(64)))
            .is_none_or(|s| {
                s[(y.rem_euclid(64) * 64 + x.rem_euclid(64)) as usize] & (1 << bit) != 0
            })
    }
    fn cardinal(&self, x: i32, y: i32, dx: i32, dy: i32) -> bool {
        match (dx, dy) {
            (0, -1) => !self.edge(x, y, 0),
            (1, 0) => !self.edge(x, y, 1),
            (0, 1) => !self.edge(x, y + 1, 0),
            (-1, 0) => !self.edge(x - 1, y, 1),
            _ => false,
        }
    }
    pub fn standing(&self, x: i32, y: i32) -> bool {
        self.contains(x, y)
            && [0, 2, 4, 6].iter().any(|&h| {
                let (dx, dy) = DIR[h];
                self.cardinal(x, y, dx, dy)
            })
    }
    pub fn can_step(&self, x: i32, y: i32, heading: u8) -> bool {
        self.can_step_with_edges(x, y, heading, |_, _, _| None)
    }
    /// S_CHANGE_ATTR replaces one Layer3 edge. Merge before evaluating either
    /// diagonal path; two independent static/dynamic checks can admit a corner.
    pub fn can_step_with_edges(
        &self, x: i32, y: i32, heading: u8,
        edge_override: impl Fn(u16, u16, u8) -> Option<bool>,
    ) -> bool {
        let Some(&(dx, dy)) = DIR.get(usize::from(heading)) else {
            return false;
        };
        let loaded = |x, y| (0..=65535).contains(&x) && (0..=65535).contains(&y)
            && self.contains(x, y);
        let cardinal = |x, y, dx, dy| {
            if !loaded(x, y) || !loaded(x + dx, y + dy) { return false; }
            let (ex, ey, bit) = match (dx, dy) {
                (0, -1) => (x, y, 0),
                (1, 0) => (x, y, 1),
                (0, 1) => (x, y + 1, 0),
                (-1, 0) => (x - 1, y, 1),
                _ => return false,
            };
            !edge_override(ex as u16, ey as u16, bit)
                .unwrap_or_else(|| self.edge(ex, ey, bit))
        };
        if !loaded(x, y) || !loaded(x + dx, y + dy)
            || ![0, 2, 4, 6].iter().any(|&h| {
                let (sx, sy) = DIR[h];
                cardinal(x + dx, y + dy, sx, sy)
            }) {
            return false;
        }
        if dx == 0 || dy == 0 {
            return cardinal(x, y, dx, dy);
        }
        (cardinal(x, y, dx, 0) && cardinal(x + dx, y, 0, dy))
            || (cardinal(x, y, 0, dy) && cardinal(x, y + dy, dx, 0))
    }
    pub fn mask(&self, x: i32, y: i32) -> u8 {
        (0..8).fold(0, |m, h| {
            m | if self.can_step(x, y, h) { 1 << h } else { 0 }
        })
    }
    /// Bounded loopback-fixture route; dynamic occupancy remains server-owned.
    pub fn route_to_adjacent(&self, start: [u16; 2], target: [u16; 2]) -> Result<Vec<u8>> {
        self.route_to_adjacent_avoiding(start,target,|_,_|false)
    }
    pub fn route_to_adjacent_avoiding(&self,start:[u16;2],target:[u16;2],blocked:impl Fn(u16,u16)->bool)->Result<Vec<u8>> {
        self.route_to_range_avoiding(start,target,1,blocked)
    }
    /// Reach a passable cell within a qualified attack range. The target may
    /// stand in terrain the local actor cannot enter; it is never a route node.
    pub fn native_line_of_sight(&self,source:[u16;2],target:[u16;2])->bool {
        let (mut x,mut y)=(i32::from(source[0]),i32::from(source[1]));let (tx,ty)=(i32::from(target[0]),i32::from(target[1]));
        if !self.standing(x,y)||!self.standing(tx,ty){return false;}
        let (dx,dy)=((tx-x).abs(),(ty-y).abs());let (sx,sy)=((tx-x).signum(),(ty-y).signum());let mut error=dx-dy;
        while x!=tx||y!=ty {let (mut nx,mut ny)=(x,y);let doubled=error*2;if doubled> -dy{error-=dy;nx+=sx;}if doubled<dx{error+=dx;ny+=sy;}
            let Some(h)=DIR.iter().position(|d|*d==(nx-x,ny-y))else{return false;};if !self.can_step(x,y,h as u8){return false;}x=nx;y=ny;
        }true
    }
    pub fn route_to_attack_range_avoiding(&self,start:[u16;2],target:[u16;2],range:u16,blocked:impl Fn(u16,u16)->bool)->Result<Vec<u8>> {
        self.route_to_range_with_goal(start,target,range,blocked,|x,y|self.native_line_of_sight([x,y],target))
    }
    pub fn route_to_range_avoiding(&self,start:[u16;2],target:[u16;2],range:u16,blocked:impl Fn(u16,u16)->bool)->Result<Vec<u8>> {
        self.route_to_range_with_goal(start,target,range,blocked,|_,_|true)
    }
    fn route_to_range_with_goal(&self,start:[u16;2],target:[u16;2],range:u16,blocked:impl Fn(u16,u16)->bool,valid:impl Fn(u16,u16)->bool)->Result<Vec<u8>> {
        self.route_to_range_bounded(start,target,range,blocked,valid,128,4096)
    }
    /// Long map entrance approach is bounded by verified geometry cells. It
    /// cannot route to an invented source or choose a server destination.
    pub fn route_to_map_entrance_avoiding(&self,start:[u16;2],source:[u16;2],range:u16,blocked:impl Fn(u16,u16)->bool)->Result<Vec<u8>> {
        ensure!(self.has_portal_source(source)&&(1..=3).contains(&range),"unknown or distant-range map entrance");
        if start[0].abs_diff(source[0]).max(start[1].abs_diff(source[1]))<=range{return Ok(Vec::new());}
        let cells=self.sectors.len().checked_mul(4096).context("map route cell budget")?;
        ensure!(cells>0&&cells<=64*4096,"map route geometry budget");
        self.route_to_range_bounded(start,source,range,blocked,|_,_|true,u32::try_from(cells)?,cells)
    }
    fn route_to_range_bounded(&self,start:[u16;2],target:[u16;2],range:u16,blocked:impl Fn(u16,u16)->bool,valid:impl Fn(u16,u16)->bool,max_steps:u32,max_cells:usize)->Result<Vec<u8>> {
        ensure!((1..=19).contains(&range),"combat route range");
        // A* keeps distant honor-arena spawns within the same bounded search;
        // breadth-first expansion spent the budget on unrelated nearby cells.
        let origin=(i32::from(start[0]),i32::from(start[1]));
        let goal=(i32::from(target[0]),i32::from(target[1]));
        let heuristic=|x:i32,y:i32| (x-goal.0).abs().max((y-goal.1).abs()).saturating_sub(i32::from(range)).max(0) as u32;
        let mut queue=BinaryHeap::from([Reverse((heuristic(origin.0,origin.1),0u32,origin))]);
        let mut costs=BTreeMap::from([(origin,0u32)]);
        let mut parents=BTreeMap::new();
        while let Some(Reverse((_,cost,(x,y))))=queue.pop() {
            if costs.get(&(x,y))!=Some(&cost){continue;}
            if (x-goal.0).abs().max((y-goal.1).abs())>0 && (x-goal.0).abs().max((y-goal.1).abs())<=i32::from(range) && valid(x as u16,y as u16) {
                let mut path=Vec::new();let mut cell=(x,y);
                while cell!=origin {let (previous,h)=parents[&cell];path.push(h);cell=previous;}
                path.reverse();return Ok(path);
            }
            if cost>=max_steps{continue;}
            for h in 0..8 {
                let (dx, dy) = DIR[h as usize];
                if self.can_step(x, y, h)
                    && (0..=65535).contains(&(x+dx)) && (0..=65535).contains(&(y+dy))
                    && !blocked((x+dx) as u16,(y+dy) as u16)
                    && (x+dx,y+dy)!=goal
                    && costs.get(&(x+dx,y+dy)).is_none_or(|old|cost+1<*old)
                {
                    let next=(x+dx,y+dy);
                    costs.insert(next,cost+1);ensure!(costs.len()<=max_cells,"route search cell budget");
                    parents.insert(next,((x,y),h));
                    queue.push(Reverse((cost+1+heuristic(next.0,next.1),cost+1,next)));
                }
            }
        }
        anyhow::bail!("no bounded combat route")
    }
    /// Bounded test route to the nearest native wall, used only by loopback probe.
    pub fn wall_route(&self, start: [u16; 2]) -> Result<(Vec<u8>, u8)> {
        let mut queue = VecDeque::from([(i32::from(start[0]), i32::from(start[1]), Vec::new())]);
        let mut seen = BTreeSet::new();
        while let Some((x, y, path)) = queue.pop_front() {
            if !seen.insert((x, y)) {
                continue;
            }
            ensure!(seen.len() <= 4096, "wall route budget");
            for h in 0..8 {
                let (dx, dy) = DIR[h as usize];
                if !path.is_empty() && self.contains(x + dx, y + dy) && !self.can_step(x, y, h) {
                    return Ok((path, h));
                }
            }
            if path.len() < 8 {
                for h in 0..8 {
                    if self.can_step(x, y, h) {
                        let (dx, dy) = DIR[h as usize];
                        let mut next = path.clone();
                        next.push(h);
                        queue.push_back((x + dx, y + dy, next));
                    }
                }
            }
        }
        anyhow::bail!("no nearby wall")
    }
}
pub fn heading(key: usize) -> Option<u8> {
    match key {
        0x26 | 0x69 => Some(1),
        0x27 | 0x63 => Some(3),
        0x28 | 0x61 => Some(5),
        0x25 | 0x67 => Some(7),
        0x68 => Some(0),
        0x66 => Some(2),
        0x62 => Some(4),
        0x64 => Some(6),
        _ => None,
    }
}
/// A physical press cannot cross a new actor life or a server interruption,
/// even when an apply/release pair arrives before the next input sample.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct InputStamp {
    epoch:u64, generation:u64, corrections:u64, locks:u64, interruptions:u64,
    movement_lock:super::native_world::MovementLock,
    actor:Option<(u32,u64,bool)>, result:Option<i32>,
}
impl InputStamp {
    pub fn capture(epoch:u64,w:&super::native_world::World)->Self {
        Self{epoch,generation:w.generation,corrections:w.local_corrections,
            locks:w.movement_locks,interruptions:w.control_interruptions,
            movement_lock:w.movement_lock,
            actor:w.own_actor().map(|a|(a.id,a.incarnation,a.dead)),result:w.battle_result}
    }
}
/// Presentation-accepted arrival hurt on the local body. A normal arrow
/// (S_ATTACK flight 0x7f, graphics 66, damage>0) defers its hurt to the
/// projectile arrival and advances no authoritative action sequence, so the
/// scene counts accepted local arrivals. The count never resets, even when the
/// scene model is replaced, so equal stamps always mean "no new hurt".
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct ImpactStamp { actor:u32, incarnation:u64, count:u64 }
impl ImpactStamp {
    pub fn record(&mut self, actor:&super::native_world::Actor) {
        self.actor=actor.id; self.incarnation=actor.incarnation;
        self.count=self.count.wrapping_add(1);
    }
    pub fn count(&self)->u64 {self.count}
}
#[derive(Default)]
pub struct HeldInput {
    keys: Vec<(usize, u8)>,
    pending: Option<u8>,
    stamp:Option<InputStamp>,
    action_stamp: Option<(u32, u64, u64)>,
    impact_stamp: Option<ImpactStamp>,
}
impl HeldInput {
    /// A presented hurt arrival on the local body interrupts walking exactly
    /// like an authoritative S_DO_ACTION hurt. The first observed stamp only
    /// establishes the baseline; any later change retires the press.
    pub fn synchronize_impact(&mut self, next: ImpactStamp) {
        if self.impact_stamp.is_some_and(|old| old != next) { self.clear(); }
        self.impact_stamp = Some(next);
    }
    /// An authoritative action interrupts walking without cancelling the
    /// independent held-attack intent. Prediction and speed changes do not.
    pub fn synchronize_action(&mut self, world: &super::native_world::World) {
        let next = world.own_actor().map(|a| (a.id, a.incarnation, a.action_sequence));
        if self.action_stamp.is_some() && self.action_stamp != next { self.clear(); }
        self.action_stamp = next;
    }
    pub fn synchronize(&mut self,stamp:Option<InputStamp>)->bool {
        let changed=self.stamp!=stamp;
        if changed || stamp.is_none() {self.clear();}
        self.stamp=stamp;changed
    }
    pub fn press(&mut self, key: usize) {
        if let Some(h) = heading(key) {
            if !self.keys.iter().any(|v| v.0 == key) {
                self.keys.push((key, h));
                self.pending = Some(h);
            }
        }
    }
    pub fn release(&mut self, key: usize) {
        self.keys.retain(|v| v.0 != key);
    }
    pub fn direction(&self) -> Option<u8> {
        self.pending.or_else(|| self.keys.last().map(|v| v.1))
    }
    pub fn sent(&mut self) {
        self.pending = None;
    }
    pub fn clear(&mut self) {
        self.keys.clear();
        self.pending = None;
    }
}
