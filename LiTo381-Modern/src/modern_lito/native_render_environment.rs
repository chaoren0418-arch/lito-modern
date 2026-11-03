//! Immutable render binding for one GAME owner and effective visual state.
use super::{native_environment::State, native_world::World};
use anyhow::{ensure, Context, Result};

/// Packet counters and raw clock seconds do not invalidate identical pixels.
/// Keep the captured authority revision separately in frame diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Key {
    pub epoch: u64,
    pub map: u16,
    pub mask_argument: u8,
    pub underwater: bool,
    pub blind: u8,
    pub ambient_level: Option<u8>,
    pub weather: [u8; 3],
}
impl Key {
    pub fn capture(connection: u64, world: &World, environment: &State) -> Result<Self> {
        ensure!(connection != 0 && environment.epoch == Some(connection), "retired render environment owner");
        ensure!(environment.water_clock.owner()==Some(connection)
            && environment.water_clock.core.enabled==environment.underwater,"render water clock owner/state mismatch");
        let map = world.map_id.context("render world map absent")?;
        ensure!(environment.map == Some(map), "render world/environment map mismatch");
        let mask_argument = environment.map_mask_argument.context("render mask policy absent")?;
        ensure!(mask_argument <= 15 && environment.blind <= 2
            && environment.ambient_level.is_none_or(|v| v <= 15)
            && environment.weather.iter().all(|v| *v <= 3), "unqualified render environment domain");
        Ok(Self { epoch: connection, map, mask_argument, underwater: environment.underwater,
            blind: environment.blind, ambient_level: environment.ambient_level, weather: environment.weather })
    }
}

#[derive(Clone)]
pub struct Bound { key: Key, state: State }
impl Bound {
    pub fn capture(connection: u64, world: &World, environment: &State) -> Result<Self> {
        Ok(Self {key: Key::capture(connection, world, environment)?, state: environment.clone()})
    }
    pub fn key(&self) -> Key { self.key }
    pub fn state(&self) -> &State { &self.state }
    pub fn actor_green_counter(&self,world:&World)->Result<u32> {
        self.validate_world(world)?;
        self.state.water_clock.green_for(self.key.epoch,world.generation)
    }
    pub fn validate_world(&self, world: &World) -> Result<()> {
        ensure!(world.map_id == Some(self.key.map), "render binding reused for another map");
        Ok(())
    }
    pub fn is_current(&self, connection: u64, world: &World, environment: &State) -> bool {
        Key::capture(connection, world, environment).is_ok_and(|key| key == self.key)
    }
}
