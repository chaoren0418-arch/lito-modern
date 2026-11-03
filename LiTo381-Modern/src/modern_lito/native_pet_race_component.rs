//! Consumer of the sealed server Native PetRace component contract.
//! This reducer consumes already accepted movement callbacks. It provides no
//! movement permission, collision proof, server result, buff or WORLD admission.
//! The outer RUNNING/roster/deadline guard belongs to the gameplay owner.
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const CONTRACT_SHA256: &str =
    "6353EDD952D3D5EC4DD116F53B71B56539771FFD956D4654E3ACB65B214CC2D9";
pub const NAMESPACE: &str = "NATIVE_LITO_LTMAP_V1";
const LAPS: u8 = 4;
const FINISH_WINDOW: i64 = 10_000_000_000;
const PANEL_COOLDOWN: i64 = 4_000_000_000;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}
impl Zone {
    fn contains(&self, c: Cell) -> bool {
        (self.min_x..=self.max_x).contains(&c.x) && (self.min_y..=self.max_y).contains(&c.y)
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PanelType {
    None,
    Transform,
    Speed,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Panel {
    #[serde(rename = "type")]
    pub kind: PanelType,
    pub point: Cell,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub map_id: u32,
    pub heading: u8,
    pub spawns: Vec<Cell>,
    pub gates: Vec<Zone>,
    pub panels: Vec<Panel>,
}
#[derive(Deserialize)]
struct Document {
    schema: String,
    namespace: String,
    mode: u8,
    maps: Vec<Course>,
    laps: u8,
    maximum_players: u8,
}
pub struct Contract {
    courses: BTreeMap<u32, Course>,
}
impl Contract {
    /// Only this exact owner-published component is supported. File pins are
    /// provenance; they do not create live session or terrain capabilities.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= 65_536, "Native race contract size bound");
        let hash = format!("{:X}", Sha256::digest(bytes));
        ensure!(hash == CONTRACT_SHA256, "unadopted Native race contract");
        let d: Document = serde_json::from_slice(bytes)?;
        ensure!(
            d.schema == "NATIVE_PET_RACE_COMPONENT_V1"
                && d.namespace == NAMESPACE
                && d.mode == 0
                && d.laps == LAPS
                && d.maximum_players == 10,
            "Native race contract namespace/policy"
        );
        let courses: BTreeMap<_, _> = d.maps.into_iter().map(|c| (c.map_id, c)).collect();
        ensure!(
            courses.keys().copied().collect::<Vec<_>>() == [566, 569, 570],
            "unqualified Native race maps"
        );
        Ok(Self { courses })
    }
    pub fn course(&self, namespace: &str, map: u32) -> Result<&Course> {
        ensure!(namespace == NAMESPACE, "Native race namespace mismatch");
        self.courses
            .get(&map)
            .context("Native race course not adopted")
    }
    pub fn begin_component(
        &self,
        namespace: &str,
        map: u32,
        participants: &[i64],
        start: i64,
        deadline: i64,
    ) -> Result<Reducer> {
        ensure!(
            deadline >= start && !participants.is_empty() && participants.len() <= 10,
            "Native race component start/count bound"
        );
        // One-runner inputs are allowed only by the isolated server reducer;
        // they are not a production minimum-player admission decision.
        let runners = participants
            .iter()
            .map(|&id| (id, Runner::default()))
            .collect();
        Ok(Reducer {
            course: self.course(namespace, map)?.clone(),
            runners,
            start,
            deadline,
            first_finisher: None,
            finish_deadline: 0,
            finished_count: 0,
        })
    }
}
#[derive(Default)]
struct Runner {
    next_gate: usize,
    laps: u8,
    finished: bool,
    transform_ready: i64,
    speed_ready: i64,
}
pub struct Reducer {
    course: Course,
    runners: BTreeMap<i64, Runner>,
    start: i64,
    deadline: i64,
    first_finisher: Option<i64>,
    finish_deadline: i64,
    finished_count: u8,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    pub gate_advanced: bool,
    pub lap_completed: bool,
    pub completed_laps: u8,
    pub current_lap: u8,
    pub finished: bool,
    pub placement: u8,
    pub elapsed_milliseconds: i64,
    pub finish_deadline_nanos: i64,
}
impl Transition {
    fn ignored(laps: u8) -> Self {
        Self {
            gate_advanced: false,
            lap_completed: false,
            completed_laps: laps,
            current_lap: (laps + 1).min(LAPS),
            finished: false,
            placement: 0,
            elapsed_milliseconds: 0,
            finish_deadline_nanos: 0,
        }
    }
}
/// Panel opportunity only. Actual form choice and timed haste/Brave application
/// remain server-owned effects; this value never grants either locally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PanelTransition {
    pub kind: PanelType,
    pub duration_seconds: u8,
    pub trailing: bool,
}
impl PanelTransition {
    fn none() -> Self {
        Self {
            kind: PanelType::None,
            duration_seconds: 0,
            trailing: false,
        }
    }
}
impl Reducer {
    pub fn course(&self) -> &Course {
        &self.course
    }
    pub fn finish_deadline_nanos(&self) -> i64 {
        self.finish_deadline
    }
    pub fn finish_window_expired(&self, now: i64) -> bool {
        self.first_finisher.is_some() && now >= self.finish_deadline
    }
    /// Exact component behavior: the reducer itself does not enforce the outer
    /// race deadline. Callers must not reinterpret this as an accepted move.
    pub fn moved(&mut self, player: i64, from: Cell, to: Cell, now: i64) -> Transition {
        let Some(r) = self.runners.get_mut(&player) else {
            return Transition::ignored(0);
        };
        if r.finished {
            return Transition::ignored(r.laps);
        }
        let next = &self.course.gates[r.next_gate];
        if next.contains(from) || !next.contains(to) {
            return Transition::ignored(r.laps);
        }
        r.next_gate += 1;
        if r.next_gate < self.course.gates.len() {
            return Transition {
                gate_advanced: true,
                lap_completed: false,
                completed_laps: r.laps,
                current_lap: r.laps + 1,
                finished: false,
                placement: 0,
                elapsed_milliseconds: 0,
                finish_deadline_nanos: self.finish_deadline,
            };
        }
        r.next_gate = 0;
        r.laps += 1;
        r.finished = r.laps >= LAPS;
        // Match Java long subtraction, including its two's-complement wrap.
        let elapsed = (now.wrapping_sub(self.start) / 1_000_000).max(0);
        let mut placement = 0;
        if r.finished {
            self.finished_count += 1;
            placement = self.finished_count;
            if self.first_finisher.is_none() {
                self.first_finisher = Some(player);
                self.finish_deadline = now.saturating_add(FINISH_WINDOW).min(self.deadline);
            }
        }
        Transition {
            gate_advanced: true,
            lap_completed: true,
            completed_laps: r.laps,
            current_lap: (r.laps + 1).min(LAPS),
            finished: r.finished,
            placement,
            elapsed_milliseconds: elapsed,
            finish_deadline_nanos: self.finish_deadline,
        }
    }
    pub fn panel_moved(&mut self, player: i64, from: Cell, to: Cell, now: i64) -> PanelTransition {
        let progress = |r: &Runner| usize::from(r.laps) * self.course.gates.len() + r.next_gate;
        let leading = self.runners.values().map(progress).max().unwrap_or(0);
        let Some(r) = self.runners.get_mut(&player) else {
            return PanelTransition::none();
        };
        if r.finished || from == to {
            return PanelTransition::none();
        }
        let Some(panel) = self.course.panels.iter().find(|p| p.point == to) else {
            return PanelTransition::none();
        };
        let trailing = progress(r) < leading;
        let ready = match panel.kind {
            PanelType::Transform => &mut r.transform_ready,
            PanelType::Speed => &mut r.speed_ready,
            PanelType::None => return PanelTransition::none(),
        };
        if now < *ready {
            return PanelTransition::none();
        }
        *ready = now.saturating_add(PANEL_COOLDOWN);
        PanelTransition {
            kind: panel.kind,
            duration_seconds: 30,
            trailing,
        }
    }
}

#[cfg(test)]
#[path = "native_pet_race_component_tests.rs"]
mod tests;
