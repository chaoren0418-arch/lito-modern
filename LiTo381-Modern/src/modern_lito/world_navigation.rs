//! Bounded mouse-route intent. One owned CPU worker and one coalescing pending
//! target; no file access, TLS, native window calls, prediction or detached work.
//! A route advances only after the server returns its exact next sequence AND
//! target cell. Rejection, changed binding/occupancy, or cancellation stops it.
use super::{
    world_map::{WorldCell, WorldMap},
    world_path::{self, PathLimits, PlannedPath},
    world_protocol::{Actor, Frame, Phase},
};
use anyhow::{ensure, Result};
use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepIntent {
    pub match_id: u64,
    pub generation: u64,
    pub from: WorldCell,
    pub to: WorldCell,
    pub heading: u8,
    pub sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Planning,
    Walking,
    WaitingForAck,
    Arrived,
    Stopped(&'static str),
    Stopping,
}
impl Status {
    pub fn text(self) -> Option<&'static str> {
        match self {
            Self::Idle => None,
            Self::Planning => Some("正在計算滑鼠路徑…"),
            Self::Walking => Some("沿路徑移動中（每一步由伺服器確認）。"),
            Self::WaitingForAck => Some("等待伺服器確認移動…"),
            Self::Arrived => Some("已到達點選位置。"),
            Self::Stopped(reason) => Some(reason),
            Self::Stopping => Some("正在取消路徑…"),
        }
    }
}

struct Input {
    serial: u64,
    map: Arc<WorldMap>,
    frame: Arc<Frame>,
    entity: u64,
    goal: WorldCell,
    objectives: BTreeSet<WorldCell>,
    life_revision: i64,
}
impl Input {
    fn actor(&self) -> &Actor {
        self.frame
            .actors
            .iter()
            .find(|a| a.entity_id == self.entity)
            .unwrap()
    }
}
fn occupied(frame: &Frame, entity: u64) -> BTreeSet<WorldCell> {
    frame
        .actors
        .iter()
        .filter(|a| a.entity_id != entity && frame.actor_occupies(a.entity_id))
        .map(|a| a.position)
        .collect()
}
fn encounter_occupied(frame: &super::npc_protocol::Frame) -> BTreeSet<WorldCell> {
    frame
        .actors
        .iter()
        .filter(|n| n.alive())
        .map(|n| n.cell)
        .chain(frame.combat.world.quest.iter().flat_map(|q| {
            q.actors
                .iter()
                .filter(|a| a.hp > 0)
                .filter_map(|a| q.target(a.slot).map(|t| t.cell))
        }))
        .chain(
            frame
                .combat
                .world
                .time_attack
                .iter()
                .filter(|d| d.tower_hp > 0)
                .map(|d| d.tower_cell),
        )
        .collect()
}
fn combat_objectives(frame: &super::combat_protocol::Frame) -> BTreeSet<WorldCell> {
    frame
        .honor
        .iter()
        .flat_map(|h| h.objectives.iter().map(|o| o.position))
        .chain(
            frame
                .world
                .siege
                .iter()
                .flat_map(|d| super::siege_space::blockers(d)),
        )
        .collect()
}

struct Job {
    serial: u64,
    cancel: Arc<AtomicBool>,
    handle: Option<JoinHandle<std::result::Result<Route, &'static str>>>,
}
impl Job {
    fn start(input: Input) -> Result<Self> {
        let serial = input.serial;
        let cancel = Arc::new(AtomicBool::new(false));
        let check = cancel.clone();
        let handle = thread::Builder::new()
            .name("lito-world-route".into())
            .spawn(move || {
                let plan = world_path::plan(
                    &input.map,
                    input.actor().position,
                    input.goal,
                    &occupied(&input.frame, input.entity)
                        .union(&input.objectives)
                        .copied()
                        .collect(),
                    PathLimits::default(),
                    &check,
                )
                .map_err(|error| match error {
                    world_path::PathError::GoalOccupied => "點選位置有其他角色，已停止移動。",
                    world_path::PathError::GoalOutside => "點選位置不在已載入的地圖範圍。",
                    world_path::PathError::GoalBlocked | world_path::PathError::NoPath => {
                        "目前沒有可通行的路徑。"
                    }
                    world_path::PathError::SearchBudgetExceeded
                    | world_path::PathError::PathTooLong { .. } => {
                        "路徑超過搜尋上限，請點選較近的位置。"
                    }
                    world_path::PathError::Cancelled => "路徑已取消。",
                    _ => "路徑資料無法確認，已停止移動。",
                })?;
                ensure_not_cancelled(&check)?;
                Ok(Route {
                    progress: Progress::new(&input.frame, input.actor(), &plan),
                    input,
                    plan,
                    offered: None,
                })
            })?;
        Ok(Self {
            serial,
            cancel,
            handle: Some(handle),
        })
    }
    fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
    }
    fn finished(&self) -> bool {
        self.handle.as_ref().is_none_or(JoinHandle::is_finished)
    }
    fn join(&mut self) -> std::result::Result<Route, &'static str> {
        self.handle
            .take()
            .ok_or("路徑工作已結束。")?
            .join()
            .unwrap_or(Err("路徑工作異常，已停止移動。"))
    }
}
fn ensure_not_cancelled(cancel: &AtomicBool) -> std::result::Result<(), &'static str> {
    if cancel.load(Ordering::Acquire) {
        Err("路徑已取消。")
    } else {
        Ok(())
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
        if self.handle.is_some() {
            let _ = self.join();
        }
    }
}

/// Sequence/cooldown authority is intentionally separate from CPU planning.
/// `offered` is not `submitted`: a locally refused command cannot advance it.
struct Progress {
    match_id: u64,
    generation: u64,
    index: usize,
    expected_sequence: u64,
    interval: Duration,
    awaiting: Option<StepIntent>,
    next_allowed: Option<Instant>,
}
impl Progress {
    fn new(frame: &Frame, actor: &Actor, _plan: &PlannedPath) -> Self {
        Self {
            match_id: frame.match_id,
            generation: frame.generation,
            index: 0,
            expected_sequence: actor.sequence,
            interval: Duration::from_nanos(actor.step_interval_nanos),
            awaiting: None,
            next_allowed: None,
        }
    }
    fn observe(
        &mut self,
        actor: &Actor,
        cells: &[WorldCell],
        now: Instant,
    ) -> std::result::Result<(), &'static str> {
        if actor.step_interval_nanos != self.interval.as_nanos() as u64 {
            return Err("伺服器移動間隔已變更，請重新點選。");
        }
        if let Some(sent) = self.awaiting {
            if actor.sequence == self.expected_sequence && actor.position == sent.from {
                return Ok(()); // Still waiting; never advance on elapsed wall time.
            }
            if actor.sequence != sent.sequence
                || actor.position != sent.to
                || actor.heading != sent.heading
            {
                return Err("伺服器未確認這一步，已停止路徑。");
            }
            self.expected_sequence = sent.sequence;
            self.index += 1;
            self.awaiting = None;
            // Conservatively rate-limit from receipt, not an assumed RTT or a
            // client timestamp. This can be slower, never faster than authority.
            self.next_allowed = Some(now.checked_add(self.interval).ok_or("移動計時超出範圍。")?);
        }
        if actor.sequence != self.expected_sequence
            || cells.get(self.index) != Some(&actor.position)
        {
            return Err("角色位置已變更，請重新點選。");
        }
        Ok(())
    }
    fn next_with_interval(
        &self,
        cells: &[WorldCell],
        headings: &[u8],
        now: Instant,
        effective: Duration,
    ) -> std::result::Result<Option<StepIntent>, &'static str> {
        // next_allowed stores the original base deadline. Recover receipt so a
        // confirmed haste/expiry change preserves elapsed progress, never reset
        // cooldown merely because the user requested another target.
        let deadline = self
            .next_allowed
            .and_then(|end| end.checked_sub(self.interval))
            .and_then(|receipt| receipt.checked_add(effective));
        if self.awaiting.is_some()
            || self.index == headings.len()
            || deadline.is_some_and(|end| now < end)
        {
            return Ok(None);
        }
        let sequence = self
            .expected_sequence
            .checked_add(1)
            .filter(|n| *n <= i64::MAX as u64)
            .ok_or("移動序號已耗盡。")?;
        Ok(Some(StepIntent {
            match_id: self.match_id,
            generation: self.generation,
            from: cells[self.index],
            to: cells[self.index + 1],
            heading: headings[self.index],
            sequence,
        }))
    }
}
struct Route {
    input: Input,
    plan: PlannedPath,
    progress: Progress,
    offered: Option<StepIntent>,
}
impl Route {
    fn advance(
        &mut self,
        frame: &Frame,
        entity: u64,
        now: Instant,
    ) -> std::result::Result<(Status, Option<StepIntent>), &'static str> {
        self.advance_with_interval(frame, entity, now, self.progress.interval)
    }
    fn advance_with_interval(
        &mut self,
        frame: &Frame,
        entity: u64,
        now: Instant,
        effective: Duration,
    ) -> std::result::Result<(Status, Option<StepIntent>), &'static str> {
        if entity != self.input.entity
            || frame.match_id != self.input.frame.match_id
            || frame.generation != self.input.frame.generation
            || frame.manifest != self.input.frame.manifest
            || frame.room != self.input.frame.room
            || frame.revision < self.input.frame.revision
            || frame.phase != Phase::Active
            || !frame.actor_occupies(entity)
        {
            return Err("遊戲或角色狀態已變更，路徑已取消。");
        }
        if super::spell_protocol::teleported(&self.input.frame, frame, entity) {
            return Err("伺服器已確認瞬間移動，原路徑已取消。");
        }
        let actor = frame
            .actors
            .iter()
            .find(|a| a.entity_id == entity && a.identity() == self.input.actor().identity())
            .ok_or("角色已不在原遊戲中。")?;
        self.progress.observe(actor, self.plan.cells(), now)?;
        let next = self.progress.next_with_interval(
            self.plan.cells(),
            self.plan.headings(),
            now,
            effective,
        )?;
        if let Some(next) = next {
            if !world_path::can_step(
                &self.input.map,
                next.from,
                next.heading,
                &occupied(frame, entity)
                    .union(&self.input.objectives)
                    .copied()
                    .collect(),
            ) {
                return Err("前方路徑已受阻，請重新點選。");
            }
        }
        self.offered = next;
        Ok((
            if self.progress.awaiting.is_some() {
                Status::WaitingForAck
            } else if self.progress.index == self.plan.steps() {
                Status::Arrived
            } else {
                Status::Walking
            },
            next,
        ))
    }
}

#[derive(Default)]
pub struct Navigation {
    serial: u64,
    pending: Option<Input>,
    job: Option<Job>,
    route: Option<Route>,
    status: Option<Status>,
}
impl Navigation {
    /// Pure bounded validation + pointer submission. Worker creation/planning
    /// occurs in poll, which owns at most one live handle at a time.
    pub fn request(
        &mut self,
        map: Arc<WorldMap>,
        frame: Arc<Frame>,
        entity: u64,
        goal: WorldCell,
    ) -> Result<()> {
        ensure!(
            !frame
                .descriptor
                .is_some_and(|d| d.has(super::world_protocol::MatchDescriptor::NPC)),
            "NPC routing requires its full roster"
        );
        ensure!(
            frame.siege.is_none(),
            "siege routing requires confirmed combat and life"
        );
        self.request_bound(map, frame, entity, goal, BTreeSet::new(), 0)
    }
    pub fn request_combat(
        &mut self,
        map: Arc<WorldMap>,
        frame: Arc<super::combat_protocol::Frame>,
        entity: u64,
        goal: WorldCell,
    ) -> Result<()> {
        frame.validate()?;
        ensure!(
            !frame
                .world
                .descriptor
                .is_some_and(|d| d.has(super::world_protocol::MatchDescriptor::NPC)),
            "NPC routing requires its full roster"
        );
        ensure!(
            frame.vital(entity).is_some_and(|v| v.hp > 0),
            "dead actor cannot route"
        );
        let objectives = combat_objectives(&frame);
        let life = super::quest_life::respawn_revision(&frame, entity);
        self.request_bound(
            map,
            Arc::new(frame.world.clone()),
            entity,
            goal,
            objectives,
            life,
        )
    }
    pub fn request_npc(
        &mut self,
        map: Arc<WorldMap>,
        reply: Arc<super::npc_protocol::Reply>,
        entity: u64,
        goal: WorldCell,
    ) -> Result<()> {
        reply.validate()?;
        let frame = reply
            .frame
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NPC snapshot absent"))?;
        ensure!(
            frame.resources.is_some() && frame.combat.vital(entity).is_some_and(|v| v.hp > 0),
            "NPC route requires live qualified actor"
        );
        self.request_bound(
            map,
            Arc::new(frame.combat.world.clone()),
            entity,
            goal,
            encounter_occupied(frame),
            super::quest_life::respawn_revision(&frame.combat, entity),
        )
    }
    fn request_bound(
        &mut self,
        map: Arc<WorldMap>,
        frame: Arc<Frame>,
        entity: u64,
        goal: WorldCell,
        objectives: BTreeSet<WorldCell>,
        life_revision: i64,
    ) -> Result<()> {
        let actor = frame
            .actors
            .iter()
            .find(|a| a.entity_id == entity)
            .ok_or_else(|| anyhow::anyhow!("route actor missing"))?;
        frame.validate(
            actor
                .identity()
                .character()
                .ok_or_else(|| anyhow::anyhow!("selected actor is not a human"))?,
        )?;
        ensure!(
            frame.phase == Phase::Active
                && frame.actor_occupies(entity)
                && frame.manifest.digest == map.canonical_digest()
                && frame.manifest.map_id == map.map_id(),
            "route requires active qualified map"
        );
        let serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("route generation exhausted"))?;
        self.cancel();
        self.serial = serial;
        self.pending = Some(Input {
            serial,
            map,
            frame,
            entity,
            goal,
            objectives,
            life_revision,
        });
        self.status = Some(Status::Planning);
        Ok(())
    }
    /// Nonblocking: join only handles reporting finished. Superseded results
    /// can never publish over the one latest pending target.
    pub fn poll(&mut self) {
        if self.job.as_ref().is_some_and(Job::finished) {
            let mut job = self.job.take().unwrap();
            let result = job.join();
            if !job.cancel.load(Ordering::Acquire)
                && job.serial == self.serial
                && self.pending.is_none()
            {
                match result {
                    Ok(route) => {
                        self.route = Some(route);
                        self.status = Some(Status::Walking);
                    }
                    Err(reason) => self.status = Some(Status::Stopped(reason)),
                }
            }
        }
        if self.job.is_none() {
            if let Some(input) = self.pending.take() {
                match Job::start(input) {
                    Ok(job) => self.job = Some(job),
                    Err(_) => self.status = Some(Status::Stopped("無法開始尋路工作。")),
                }
            } else if self.status == Some(Status::Stopping) {
                self.status = Some(Status::Idle);
            }
        }
    }
    pub fn next_step(&mut self, frame: &Frame, entity: u64, now: Instant) -> Option<StepIntent> {
        if frame.siege.is_some()
            || frame
                .descriptor
                .is_some_and(|d| d.has(super::world_protocol::MatchDescriptor::NPC))
        {
            self.cancel();
            return None;
        }
        let result = self.route.as_mut()?.advance(frame, entity, now);
        self.accept_advance(result)
    }
    pub fn next_step_combat(
        &mut self,
        frame: &super::combat_protocol::Frame,
        entity: u64,
        now: Instant,
    ) -> Option<StepIntent> {
        if frame
            .world
            .descriptor
            .is_some_and(|d| d.has(super::world_protocol::MatchDescriptor::NPC))
        {
            self.cancel();
            return None;
        }
        if self.route.as_ref().is_some_and(|r| {
            super::quest_life::respawn_revision(frame, entity) > r.input.life_revision
                || frame.vital(entity).is_none_or(|v| v.hp == 0)
                || combat_objectives(frame) != r.input.objectives
        }) {
            self.cancel();
            return None;
        }
        let actor = frame.world.actors.iter().find(|a| a.entity_id == entity)?;
        let interval = super::timed_protocol::effective_nanos(
            frame,
            entity,
            actor.step_interval_nanos,
            super::timed_protocol::Pace::Move,
        );
        let result = self.route.as_mut()?.advance_with_interval(
            &frame.world,
            entity,
            now,
            Duration::from_nanos(interval),
        );
        self.accept_advance(result)
    }
    pub fn next_step_npc(
        &mut self,
        reply: &super::npc_protocol::Reply,
        entity: u64,
        now: Instant,
    ) -> Option<StepIntent> {
        let frame = reply.frame.as_ref()?;
        if reply.validate().is_err()
            || frame.resources.is_none()
            || frame.combat.vital(entity).is_none_or(|v| v.hp == 0)
            || self.route.as_ref().is_some_and(|r| {
                super::quest_life::respawn_revision(&frame.combat, entity) > r.input.life_revision
            })
        {
            self.cancel();
            return None;
        }
        // Moving NPCs refresh only dynamic occupancy. An unrelated NPC move
        // need not cancel the route; an obstructed next segment still stops it.
        let route = self.route.as_mut()?;
        route.input.objectives = encounter_occupied(frame);
        let actor = frame
            .combat
            .world
            .actors
            .iter()
            .find(|a| a.entity_id == entity)?;
        let interval = super::timed_protocol::effective_nanos(
            &frame.combat,
            entity,
            actor.step_interval_nanos,
            super::timed_protocol::Pace::Move,
        );
        let result = route.advance_with_interval(
            &frame.combat.world,
            entity,
            now,
            Duration::from_nanos(interval),
        );
        self.accept_advance(result)
    }
    fn accept_advance(
        &mut self,
        result: std::result::Result<(Status, Option<StepIntent>), &'static str>,
    ) -> Option<StepIntent> {
        match result {
            Ok((status, intent)) => {
                self.status = Some(status);
                intent
            }
            Err(reason) => {
                self.route = None;
                self.status = Some(Status::Stopped(reason));
                None
            }
        }
    }
    /// Call ONLY after the single TLS owner's local command admission succeeds.
    pub fn submitted(&mut self, intent: StepIntent) -> Result<()> {
        let route = self
            .route
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("route no longer exists"))?;
        ensure!(
            route.offered == Some(intent) && route.progress.awaiting.is_none(),
            "route step was not offered or is already pending"
        );
        route.progress.awaiting = Some(intent);
        route.offered = None;
        self.status = Some(Status::WaitingForAck);
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.pending = None;
        self.route = None;
        if let Some(job) = &self.job {
            job.cancel();
        }
        self.status = Some(if self.job.is_some() {
            Status::Stopping
        } else {
            Status::Idle
        });
    }
    pub fn status(&self) -> Status {
        self.status.unwrap_or(Status::Idle)
    }
    pub fn is_quiescent(&self) -> bool {
        self.job.is_none() && self.pending.is_none()
    }
}
impl Drop for Navigation {
    fn drop(&mut self) {
        self.cancel(); /* Job::drop retains and joins its handle. */
    }
}

#[cfg(test)]
#[path = "world_navigation_tests.rs"]
mod tests;
