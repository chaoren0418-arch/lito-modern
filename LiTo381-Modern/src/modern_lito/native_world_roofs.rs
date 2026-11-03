//! Source Layer5 roof transitions driven by the current local actor's logical cell.
//! Uses elapsed-time catch-up, not original timer queue or tile-pointer alias history.
use super::{
    world_map::{WorldCell, WorldMap},
    world_roofs,
};
use anyhow::{ensure, Context, Result};
use std::time::{Duration, Instant};

pub struct Definition {
    pub(crate) map: world_roofs::RoofMap,
    digest: [u8; 32],
}
impl Definition {
    pub fn from_map(map: &WorldMap) -> Result<Self> {
        Ok(Self {
            map: world_roofs::RoofMap::new(map)?,
            digest: map.canonical_digest(),
        })
    }
    pub fn receipt(&self) -> serde_json::Value {
        self.map.receipt()
    }
}
pub struct Runtime {
    origin: Instant,
    last_ns: Option<u64>,
    identity: Option<[u64; 3]>,
    definition: Option<[u8; 32]>,
    animation: world_roofs::Animation,
}
impl Default for Runtime {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
            last_ns: None,
            identity: None,
            definition: None,
            animation: Default::default(),
        }
    }
}
#[derive(Default)]
pub struct Frame {
    pub(crate) shades: world_roofs::Frame,
    report: serde_json::Value,
    definition: Option<[u8; 32]>,
}
impl Frame {
    pub(crate) fn matches_definition(&self, d: &Definition) -> bool {
        self.definition.is_none_or(|v| v == d.digest)
    }
    pub fn receipt(&self) -> &serde_json::Value {
        &self.report
    }
}
impl Runtime {
    pub fn sample(
        &mut self,
        definition: &Definition,
        identity: [u64; 3],
        cell: WorldCell,
        elapsed_ns: u64,
    ) -> Result<Frame> {
        ensure!(identity.iter().all(|v| *v != 0), "roof identity absent");
        ensure!(
            self.last_ns.is_none_or(|t| elapsed_ns >= t),
            "roof elapsed clock moved backwards"
        );
        let now = self
            .origin
            .checked_add(Duration::from_nanos(elapsed_ns))
            .context("roof clock overflow")?;
        ensure!(
            super::world_depth::NativeDrawCell::from_world(cell).is_some(),
            "roof cell overflow"
        );
        if self.identity != Some(identity) || self.definition != Some(definition.digest) {
            self.animation = Default::default();
            self.identity = Some(identity);
            self.definition = Some(definition.digest);
        }
        let shades = self
            .animation
            .sample(&definition.map, identity[0], identity[1], cell, now)?;
        self.last_ns = Some(elapsed_ns);
        let report = serde_json::json!({"identity":identity,"geometry_sha256":definition.digest.iter().map(|v|format!("{v:02x}")).collect::<String>(),"cell":[cell.x,cell.y],"elapsed_ns":elapsed_ns,"groups":shades.receipt(),"clock":"50ms catch-up, step4, RGB555 weight0..31; stable half uses average","native_timer_queue_equivalent":false,"native_pointer_alias_equivalent":false});
        Ok(Frame {
            shades,
            report,
            definition: Some(definition.digest),
        })
    }
}
