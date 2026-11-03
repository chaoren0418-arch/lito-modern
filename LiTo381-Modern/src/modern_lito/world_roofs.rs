//! Shared S32 Layer5, not a screen-space "hide every roof" approximation.
//! 483C91 ignores serialized group IDs; 483FF4 indexes the sector's group
//! array. 483890 maps kind 0/1 to flags 1/2. 48AFC0 uses the local actor's
//! exact native half-cell, resetting the old groups and applying the new ones.
//! 432F70/4330D0 change a 0..31 source weight by four every 50ms. Stable
//! half-opacity uses the separate RGB555 average, not weight 16/31.
//! The modern clock catches up elapsed quanta and reverses from the current
//! weight. It does not emulate native timer-queue delays or pointer aliasing
//! when different groups reference the same tile in the same cell.
use super::{
    world_depth::NativeDrawCell,
    world_map::{WorldCell, WorldMap},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};

const MAX_RECORDS: usize = 262_144;
const MAX_GROUPS: usize = 65_536;
const QUANTUM: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Group(pub u16, pub u16, pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Half,
    Hidden,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) enum Shade {
    Native,
    Half,
    Hidden,
    Weight(u8),
}

#[derive(Default)]
pub(crate) struct RoofMap {
    triggers: BTreeMap<(i32, i32), Vec<(Group, Mode)>>,
    groups: BTreeSet<Group>,
    pub ignored_outside_sector: usize,
}
impl RoofMap {
    pub(crate) fn receipt(&self) -> serde_json::Value {
        serde_json::json!({"trigger_cells":self.triggers.len(),"controlled_groups":self.groups.len(),"ignored_outside_sector":self.ignored_outside_sector})
    }

    pub fn new(map: &WorldMap) -> Result<Self> {
        let mut result = Self::default();
        let mut records = 0usize;
        let mut groups = 0usize;
        for binding in map.bindings() {
            let sector = map.sector(binding.key).context("roof sector absent")?;
            records = records
                .checked_add(sector.opacity.len())
                .context("roof record overflow")?;
            groups = groups
                .checked_add(sector.objects.len())
                .context("roof group overflow")?;
            ensure!(
                records <= MAX_RECORDS && groups <= MAX_GROUPS,
                "roof metadata budget exceeded"
            );
            let (x, y) = binding.key.world_origin();
            let origin =
                NativeDrawCell::from_world(WorldCell { x, y }).context("roof origin overflow")?;
            for record in &sector.opacity {
                // The loader checks coordinates before dereferencing the group.
                // Shared files retain off-sector duplicate metadata; do not wrap
                // it into an adjacent sector or reinterpret odd X as even X.
                if record.x >= 128 || record.y >= 64 {
                    result.ignored_outside_sector += 1;
                    continue;
                }
                ensure!(
                    usize::from(record.object_index) < sector.objects.len(),
                    "roof group index outside sector"
                );
                let mode = match record.kind {
                    0 => Mode::Half,
                    1 => Mode::Hidden,
                    _ => anyhow::bail!("unqualified Layer5 kind {}", record.kind),
                };
                let group = Group(
                    binding.key.sector_x,
                    binding.key.sector_y,
                    record.object_index,
                );
                result.groups.insert(group);
                result
                    .triggers
                    .entry((
                        origin.half_x + i32::from(record.x),
                        origin.y + i32::from(record.y),
                    ))
                    .or_default()
                    .push((group, mode));
            }
        }
        Ok(result)
    }
    pub fn controls(&self, group: Group) -> bool {
        self.groups.contains(&group)
    }
    fn targets(&self, cell: WorldCell) -> Result<BTreeMap<Group, Mode>> {
        let cell = NativeDrawCell::from_world(cell).context("roof actor origin overflow")?;
        // Last source record wins if the same group has multiple kinds at one
        // cell, as sequential native 433210 calls replace the target flags.
        Ok(self
            .triggers
            .get(&(cell.half_x, cell.y))
            .into_iter()
            .flatten()
            .copied()
            .collect())
    }
    pub fn settled(&self, cell: WorldCell) -> Result<Frame> {
        Ok(Frame(
            self.targets(cell)?
                .into_iter()
                .map(|(group, mode)| {
                    let shade = match mode {
                        Mode::Half => Shade::Half,
                        Mode::Hidden => Shade::Hidden,
                    };
                    (group, [shade; 2])
                })
                .collect(),
        ))
    }
}

#[derive(Default)]
pub(crate) struct Frame(BTreeMap<Group, [Shade; 2]>);
impl Frame {
    pub(crate) fn receipt(&self) -> serde_json::Value {
        serde_json::json!(self.0.iter().map(|(g,s)| serde_json::json!({"sector":[g.0,g.1],"ordinal":g.2,"normal":s[0],"base_half":s[1]})).collect::<Vec<_>>())
    }

    pub fn shade(&self, group: Option<Group>, base_half: bool) -> Shade {
        group
            .and_then(|g| self.0.get(&g))
            .map_or(Shade::Native, |s| s[usize::from(base_half)])
    }
}

#[derive(Clone, Copy, Debug)]
struct Ramp {
    shade: Shade,
    target: Shade,
    at: Instant,
}
impl Ramp {
    fn new(base_half: bool, now: Instant) -> Self {
        let shade = if base_half {
            Shade::Half
        } else {
            Shade::Native
        };
        Self {
            shade,
            target: shade,
            at: now,
        }
    }
    fn advance(&mut self, now: Instant) {
        let steps = now.duration_since(self.at).as_nanos() / QUANTUM.as_nanos();
        // All transitions settle in <= 9 steps. Long background pauses do not
        // cause an unbounded loop or restart an already completed transition.
        for _ in 0..steps.min(9) {
            let Shade::Weight(weight) = self.shade else {
                break;
            };
            self.shade = match self.target {
                Shade::Hidden if weight < 4 => Shade::Hidden,
                Shade::Hidden => Shade::Weight(weight - 4),
                Shade::Half if weight > 16 && weight < 21 => Shade::Half,
                Shade::Half if weight > 16 => Shade::Weight(weight - 4),
                Shade::Half if weight > 12 => Shade::Half,
                Shade::Half => Shade::Weight(weight + 4),
                Shade::Native if weight > 27 => Shade::Native,
                Shade::Native => Shade::Weight(weight + 4),
                Shade::Weight(_) => unreachable!(),
            };
            self.at += QUANTUM;
        }
    }
    fn target(&mut self, target: Shade, now: Instant) {
        self.advance(now);
        if self.target == target {
            return;
        }
        self.target = target;
        if self.shade != target {
            self.shade = match self.shade {
                Shade::Native => Shade::Weight(31),
                Shade::Half => Shade::Weight(if target == Shade::Native { 17 } else { 16 }),
                Shade::Hidden => Shade::Weight(1),
                shade @ Shade::Weight(_) => shade,
            };
        }
        self.at = now;
    }
}

#[derive(Default)]
pub(crate) struct Animation {
    identity: Option<(u64, u64)>,
    last: Option<Instant>,
    groups: BTreeMap<Group, [Ramp; 2]>,
}
impl Animation {
    pub fn sample(
        &mut self,
        map: &RoofMap,
        world: u64,
        selected: u64,
        cell: WorldCell,
        now: Instant,
    ) -> Result<Frame> {
        ensure!(world != 0 && selected != 0, "roof scope absent");
        ensure!(
            self.last.is_none_or(|last| now >= last),
            "roof clock moved backwards"
        );
        let targets = map.targets(cell)?;
        if self.identity != Some((world, selected)) {
            self.groups.clear();
            self.identity = Some((world, selected));
        }
        self.last = Some(now);
        for &group in targets.keys() {
            self.groups
                .entry(group)
                .or_insert_with(|| [Ramp::new(false, now), Ramp::new(true, now)]);
        }
        for (group, ramps) in &mut self.groups {
            for (base_half, ramp) in ramps.iter_mut().enumerate() {
                let target = match targets.get(group) {
                    Some(Mode::Half) => Shade::Half,
                    Some(Mode::Hidden) => Shade::Hidden,
                    None if base_half == 1 => Shade::Half,
                    None => Shade::Native,
                };
                ramp.target(target, now);
            }
        }
        self.groups.retain(|group, ramps| {
            targets.contains_key(group)
                || ramps[0].shade != Shade::Native
                || ramps[1].shade != Shade::Half
        });
        Ok(Frame(
            self.groups
                .iter()
                .map(|(&group, ramps)| (group, [ramps[0].shade, ramps[1].shade]))
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{resources::SharedMapSectorKey, world_map::SectorInput};
    use std::sync::atomic::AtomicBool;
    fn fixture(records: &[[u8; 5]]) -> Vec<u8> {
        let mut raw = vec![0; 32768 + 2 + 16384];
        raw.extend_from_slice(&2i32.to_le_bytes());
        // Deliberately nonordinal IDs: native linkage must not use 70/-7.
        for id in [70i16, -7] {
            raw.extend_from_slice(&id.to_le_bytes());
            raw.extend_from_slice(&0u16.to_le_bytes());
        }
        raw.extend_from_slice(&(records.len() as i32).to_le_bytes());
        for r in records {
            raw.extend_from_slice(r);
        }
        raw
    }
    fn loaded(records: &[[u8; 5]]) -> WorldMap {
        WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: SharedMapSectorKey {
                    map_id: 514,
                    sector_x: 0x8000,
                    sector_y: 0x8000,
                },
                bytes: fixture(records),
            }],
            &AtomicBool::new(false),
        )
        .unwrap()
    }
    #[test]
    fn encoded_layer5_uses_sector_ordinals_and_rejects_unknown_live_references() {
        let m = loaded(&[
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0],
            [128, 0, 255, 255, 255],
            [0, 64, 255, 255, 255],
        ]);
        let r = RoofMap::new(&m).unwrap();
        assert_eq!(r.ignored_outside_sector, 2);
        let f = r.settled(cell()).unwrap();
        assert_eq!(
            f.shade(Some(Group(0x8000, 0x8000, 0)), false),
            Shade::Hidden
        );
        assert_eq!(f.shade(Some(Group(0x8000, 0x8000, 1)), false), Shade::Half);
        assert_eq!(
            f.shade(Some(Group(0x8000, 0x8000, 70)), false),
            Shade::Native
        );
        assert!(RoofMap::new(&loaded(&[[0, 0, 2, 0, 0]])).is_err());
        assert!(RoofMap::new(&loaded(&[[0, 0, 0, 0, 2]])).is_err());
        assert!(RoofMap::new(&loaded(&[[0, 0, 0, 0, 255]])).is_err());
    }
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; independent C# raw Layer5 canonical, no output"]
    fn shared_roof_bindings_match_independent_csharp_metadata() {
        use sha2::{Digest, Sha256};
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
        );
        for (sx, sy, source_sha, records, outside, canonical_sha) in [
            (
                0x7fff,
                0x8000,
                "ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271",
                17,
                0,
                "a90853078f29ef1f29aa1b9b11252195d2e6392eb35a6f9d9d0c036f8b76d504",
            ),
            (
                0x7ffd,
                0x7fff,
                "ac3e658f59e20bc6e5fcd4eac7cd8f08a405882ad990dd1aa82e4c5c5002c83c",
                1195,
                79,
                "31fe485e5407004127d2f96c5b910f19e3e186c1881ac83408860887b230d15a",
            ),
        ] {
            let key = SharedMapSectorKey {
                map_id: 514,
                sector_x: sx,
                sector_y: sy,
            };
            let bytes = super::super::resources::read_shared_map_sector_with_limit(
                &root,
                key,
                super::super::resources::MAX_SHARED_MAP_SECTOR_BYTES,
            )
            .unwrap();
            assert_eq!(format!("{:x}", Sha256::digest(&bytes)), source_sha);
            let m = WorldMap::from_sector_bytes(
                vec![SectorInput { key, bytes }],
                &AtomicBool::new(false),
            )
            .unwrap();
            let roofs = RoofMap::new(&m).unwrap();
            assert_eq!(m.sector(key).unwrap().opacity.len(), records);
            assert_eq!(roofs.ignored_outside_sector, outside);
            let mut canonical = Vec::new();
            for (&(hx, y), rows) in &roofs.triggers {
                for (Group(x, z, g), mode) in rows {
                    canonical.extend_from_slice(&hx.to_le_bytes());
                    canonical.extend_from_slice(&y.to_le_bytes());
                    for v in [x, z, g] {
                        canonical.extend_from_slice(&v.to_le_bytes());
                    }
                    canonical.push(if *mode == Mode::Half { 0 } else { 1 });
                }
            }
            assert_eq!(canonical.len(), 15 * (records - outside));
            assert_eq!(format!("{:x}", Sha256::digest(&canonical)), canonical_sha);
        }
    }
    fn cell() -> WorldCell {
        WorldCell { x: 32768, y: 32768 }
    }
    fn map(mode: Mode) -> RoofMap {
        RoofMap {
            triggers: BTreeMap::from([((32768, 32768), vec![(Group(1, 2, 3), mode)])]),
            ..Default::default()
        }
    }
    fn shade(f: &Frame) -> Shade {
        f.shade(Some(Group(1, 2, 3)), false)
    }
    #[test]
    fn enter_half_and_leave_preserve_fifty_ms_quanta() {
        let m = map(Mode::Half);
        let mut a = Animation::default();
        let t = Instant::now();
        for (ms, s) in [(0, 31), (49, 31), (50, 27), (100, 23), (150, 19)] {
            assert_eq!(
                shade(
                    &a.sample(&m, 1, 2, cell(), t + Duration::from_millis(ms))
                        .unwrap()
                ),
                Shade::Weight(s)
            );
        }
        assert_eq!(
            shade(
                &a.sample(&m, 1, 2, cell(), t + Duration::from_millis(200))
                    .unwrap()
            ),
            Shade::Half
        );
        let outside = WorldCell { x: 32769, ..cell() };
        for (ms, s) in [(250, 17), (299, 17), (300, 21), (350, 25), (400, 29)] {
            assert_eq!(
                shade(
                    &a.sample(&m, 1, 2, outside, t + Duration::from_millis(ms))
                        .unwrap()
                ),
                Shade::Weight(s)
            );
        }
        assert_eq!(
            shade(
                &a.sample(&m, 1, 2, outside, t + Duration::from_millis(450))
                    .unwrap()
            ),
            Shade::Native
        );
        assert!(a.groups.is_empty());
    }
    #[test]
    fn hidden_reverse_scope_and_clock_are_bounded() {
        let m = map(Mode::Hidden);
        let mut a = Animation::default();
        let t = Instant::now();
        a.sample(&m, 1, 2, cell(), t).unwrap();
        assert_eq!(
            shade(
                &a.sample(&m, 1, 2, cell(), t + Duration::from_millis(350))
                    .unwrap()
            ),
            Shade::Weight(3)
        );
        assert_eq!(
            shade(
                &a.sample(&m, 1, 2, cell(), t + Duration::from_millis(400))
                    .unwrap()
            ),
            Shade::Hidden
        );
        let outside = WorldCell { x: 32769, ..cell() };
        assert_eq!(
            shade(
                &a.sample(&m, 1, 2, outside, t + Duration::from_millis(450))
                    .unwrap()
            ),
            Shade::Weight(1)
        );
        assert_eq!(
            shade(
                &a.sample(&m, 1, 2, outside, t + Duration::from_millis(550))
                    .unwrap()
            ),
            Shade::Weight(9)
        );
        assert_eq!(
            shade(
                &a.sample(&m, 1, 2, cell(), t + Duration::from_millis(550))
                    .unwrap()
            ),
            Shade::Weight(9)
        );
        assert!(a.sample(&m, 1, 2, cell(), t).is_err());
        assert_eq!(
            shade(
                &a.sample(&m, 2, 2, outside, t + Duration::from_millis(600))
                    .unwrap()
            ),
            Shade::Native
        );
        a.sample(&m, 2, 2, cell(), t + Duration::from_millis(650))
            .unwrap();
        assert_eq!(
            shade(
                &a.sample(&m, 2, 2, cell(), t + Duration::from_secs(3600))
                    .unwrap()
            ),
            Shade::Hidden
        );
        assert_eq!(
            shade(
                &a.sample(&m, 2, 3, outside, t + Duration::from_secs(3600))
                    .unwrap()
            ),
            Shade::Native
        );
    }
    #[test]
    fn exact_half_cell_and_source_order_not_camera_or_group_id() {
        let mut m = map(Mode::Half);
        m.triggers
            .insert((32769, 32768), vec![(Group(1, 2, 4), Mode::Hidden)]);
        m.triggers
            .get_mut(&(32768, 32768))
            .unwrap()
            .push((Group(1, 2, 3), Mode::Hidden));
        let f = m.settled(cell()).unwrap();
        assert_eq!(shade(&f), Shade::Hidden);
        assert_eq!(f.shade(Some(Group(1, 2, 4)), false), Shade::Native);
        assert_eq!(f.shade(Some(Group(9, 2, 3)), false), Shade::Native);
        assert_eq!(f.shade(None, false), Shade::Native);
        assert!(m.settled(WorldCell { x: i32::MAX, y: 0 }).is_err());
    }
}
