//! Native terrain policy for the adopted three PetRace courses. Geometry is
//! parsed from pinned S32 inputs; exported Java masks are never load inputs.
//! This is terrain permission only, excluding actor occupancy and live admission.
use super::{
    native_pet_race_component::{Cell, NAMESPACE},
    native_resources::SectorPin,
    resources::SharedMapSectorKey,
    world_map::{SectorInput, WorldCell, WorldMap, DIRECTIONS},
};
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};

pub struct NativeTerrain {
    map: WorldMap,
}
fn world(c: Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
impl NativeTerrain {
    pub fn from_sector_bytes(
        namespace: &str,
        inputs: Vec<SectorInput>,
        pins: &[SectorPin],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        ensure!(namespace == NAMESPACE, "Native race terrain namespace");
        ensure!(
            !pins.is_empty() && inputs.len() == pins.len(),
            "Native terrain sector pins absent/count"
        );
        let map_id = pins[0].key.map_id;
        ensure!(
            matches!(map_id, 566 | 569 | 570) && pins.iter().all(|p| p.key.map_id == map_id),
            "Native race terrain map"
        );
        let expected: BTreeMap<_, _> = pins.iter().map(|p| ((p.key.x, p.key.y), p)).collect();
        ensure!(
            expected.len() == pins.len(),
            "Native terrain duplicate sector pin"
        );
        let map = WorldMap::from_native_sector_bytes(inputs, cancel)?;
        ensure!(map.map_id() == map_id, "Native terrain map/pin mismatch");
        for b in map.bindings() {
            let p = expected
                .get(&(b.key.sector_x, b.key.sector_y))
                .context("Native terrain unpinned sector")?;
            ensure!(
                u64::from(b.byte_length) == p.pin.bytes && b.sha256 == p.pin.sha256,
                "Native terrain sector hash/length mismatch"
            );
        }
        Ok(Self { map })
    }
    pub fn geometry(&self) -> &WorldMap {
        &self.map
    }
    pub fn contains(&self, cell: Cell) -> bool {
        self.map.contains(world(cell))
    }
    fn attribute(&self, c: Cell, attribute: usize) -> Option<bool> {
        let x = i64::from(c.x) - 32768;
        let y = i64::from(c.y) - 32768;
        let key = SharedMapSectorKey {
            map_id: self.map.map_id(),
            sector_x: u16::try_from(x.div_euclid(64) + 32768).ok()?,
            sector_y: u16::try_from(y.div_euclid(64) + 32768).ok()?,
        };
        let a = self
            .map
            .sector(key)?
            .attributes
            .get((y.rem_euclid(64) * 64 + x.rem_euclid(64)) as usize)?;
        Some(if attribute == 0 {
            !a.north_edge_open()
        } else {
            !a.east_edge_open()
        })
    }
    fn blocked(&self, c: Cell, dx: i32, dy: i32, attribute: usize) -> bool {
        let Some(x) = c.x.checked_add(dx) else {
            return true;
        };
        let Some(y) = c.y.checked_add(dy) else {
            return true;
        };
        self.attribute(Cell { x, y }, attribute).unwrap_or(true)
    }
    fn direction_blocked(&self, c: Cell, h: usize) -> bool {
        let e = |x, y, a| self.blocked(c, x, y, a);
        match h {
            0 => e(0, 0, 0),
            1 => (e(0, 0, 0) || e(0, -1, 1)) && (e(0, 0, 1) || e(1, 0, 0)),
            2 => e(0, 0, 1),
            3 => (e(0, 0, 1) || e(1, 1, 0)) && (e(0, 1, 0) || e(0, 1, 1)),
            4 => e(0, 1, 0),
            5 => (e(0, 1, 0) || e(-1, 1, 1)) && (e(-1, 0, 1) || e(-1, 1, 0)),
            6 => e(-1, 0, 1),
            7 => (e(-1, 0, 1) || e(-1, 0, 0)) && (e(-1, -1, 1) || e(0, 0, 0)),
            _ => true,
        }
    }
    pub fn standing(&self, c: Cell) -> bool {
        self.contains(c) && [0, 2, 4, 6].iter().any(|&h| !self.direction_blocked(c, h))
    }
    pub fn can_step(&self, from: Cell, to: Cell) -> bool {
        if !self.contains(from) || !self.contains(to) {
            return false;
        }
        if from == to {
            return self.standing(from);
        }
        let dx = i64::from(to.x) - i64::from(from.x);
        let dy = i64::from(to.y) - i64::from(from.y);
        DIRECTIONS
            .iter()
            .position(|&(x, y)| i64::from(x) == dx && i64::from(y) == dy)
            .is_some_and(|h| self.standing(to) && !self.direction_blocked(from, h))
    }
    pub fn direction_mask(&self, from: Cell) -> u8 {
        let mut mask = 0;
        for (h, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
            if let (Some(x), Some(y)) = (from.x.checked_add(dx), from.y.checked_add(dy)) {
                if self.can_step(from, Cell { x, y }) {
                    mask |= 1 << h;
                }
            }
        }
        mask
    }
    /// Parsed Layer3 bit0 pairs, same LSB-first packing as the actual catalog.
    pub fn packed_layer3(&self, key: SharedMapSectorKey) -> Result<[u8; 1024]> {
        let data = self
            .map
            .sector(key)
            .context("Native terrain Layer3 sector absent")?;
        ensure!(data.attributes.len() == 4096, "Native Layer3 cell count");
        let mut bits = [0; 1024];
        for (i, a) in data.attributes.iter().enumerate() {
            for (j, blocked) in [!a.north_edge_open(), !a.east_edge_open()]
                .into_iter()
                .enumerate()
            {
                if blocked {
                    let bit = i * 2 + j;
                    bits[bit / 8] |= 1 << (bit % 8);
                }
            }
        }
        Ok(bits)
    }
}

#[cfg(test)]
#[path = "native_pet_race_terrain_tests.rs"]
mod tests;
