//! Necessary composition of actual Native terrain with the adopted reducer.
//! Offline clock values and seats are explicit. This consumer sends no packets,
//! applies no server effects and does not create RUNNING/roster/WORLD authority.
use super::{
    native_pet_race_component::{Cell, Contract, PanelTransition, Reducer, Transition, NAMESPACE},
    native_pet_race_terrain::NativeTerrain,
};
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeMap, sync::Arc};

pub struct OfflineRace {
    terrain: Arc<NativeTerrain>,
    reducer: Option<Reducer>,
    positions: BTreeMap<i64, Cell>,
    last: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub from: Cell,
    pub to: Cell,
    pub panel: PanelTransition,
    pub race: Transition,
}
impl OfflineRace {
    pub fn begin(
        contract: &Contract,
        terrain: Arc<NativeTerrain>,
        seats: &[(i64, Cell)],
        start: i64,
        deadline: i64,
    ) -> Result<Self> {
        let course = contract.course(NAMESPACE, terrain.geometry().map_id())?;
        ensure!(
            !seats.is_empty() && seats.len() <= 10,
            "offline Native race seat count"
        );
        let positions: BTreeMap<_, _> = seats.iter().copied().collect();
        ensure!(
            positions.len() == seats.len()
                && seats
                    .iter()
                    .all(|(_, c)| course.spawns.contains(c) && terrain.standing(*c)),
            "offline Native race seat identity/spawn/floor"
        );
        let participants: Vec<_> = seats.iter().map(|(id, _)| *id).collect();
        let reducer =
            contract.begin_component(NAMESPACE, course.map_id, &participants, start, deadline)?;
        Ok(Self {
            terrain,
            reducer: Some(reducer),
            positions,
            last: start,
        })
    }
    pub fn position(&self, player: i64) -> Result<Cell> {
        ensure!(self.reducer.is_some(), "offline Native race retired");
        self.positions
            .get(&player)
            .copied()
            .context("offline Native race runner absent")
    }
    pub fn step(&mut self, player: i64, to: Cell, now: i64) -> Result<Step> {
        let from = self.position(player)?;
        // Modern offline clock policy, not a recovered live race deadline guard.
        ensure!(now >= self.last, "offline Native race clock reversed");
        ensure!(
            self.terrain.can_step(from, to),
            "offline Native terrain edge rejected"
        );
        let r = self
            .reducer
            .as_mut()
            .context("offline Native race retired")?;
        // Same source order as GameHub.petRaceMoved: panel before progression.
        let panel = r.panel_moved(player, from, to, now);
        let race = r.moved(player, from, to, now);
        self.positions.insert(player, to);
        self.last = now;
        Ok(Step {
            from,
            to,
            panel,
            race,
        })
    }
    pub fn retire(&mut self) {
        self.reducer = None;
        self.positions.clear();
    }
}
