//! Owned canonical ground routing. Planning is CPU-only and bounded; each
//! step waits for its own new receipt AND exact authoritative pose/sequence.
//! Cancellation never rolls back a sent command or invents a legacy Frame.
use super::{
    battle_control::{Command, Outcome},
    battle_scene::Surface,
    battle_window::{Intent, Kind, Local, Stamp},
    login_window::LoginSnapshot,
    world_map::WorldCell,
    world_navigation::Status,
    world_path::{PathLimits, PlannedPath},
};
use anyhow::{ensure, Context, Result};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

struct Input {
    navigation: Arc<super::battle_space::Navigation>,
    stamp: Stamp,
    goal: WorldCell,
}
struct Job {
    cancel: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<Walking>>>,
}
impl Job {
    fn start(input: Input) -> Result<Self> {
        let cancel = Arc::new(AtomicBool::new(false));
        let check = cancel.clone();
        let handle = thread::Builder::new()
            .name("lito-canonical-route".into())
            .spawn(move || {
                let plan = input
                    .navigation
                    .plan(input.goal, PathLimits::default(), &check)?;
                ensure!(!check.load(Ordering::Acquire), "route cancelled");
                let data = input.navigation.observation().data();
                let actor = data
                    .actors
                    .0
                    .iter()
                    .find(|a| a.entity == input.stamp.entity)
                    .context("route actor")?;
                Ok(Walking {
                    stamp: input.stamp,
                    plan,
                    index: 0,
                    sequence: actor.last_move_sequence,
                    interval: Duration::from_nanos(actor.step_interval_nanos.try_into()?),
                    next_allowed: None,
                    offered: None,
                    waiting: None,
                })
            })?;
        Ok(Self {
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
    fn join(&mut self) -> Result<Walking> {
        self.handle
            .take()
            .context("route worker retired")?
            .join()
            .map_err(|_| anyhow::anyhow!("route worker panicked"))?
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
struct Step {
    command: Command,
    prior: Option<u64>,
    to: WorldCell,
    sequence: i64,
    heading: u8,
}
struct Walking {
    stamp: Stamp,
    plan: PlannedPath,
    index: usize,
    sequence: i64,
    interval: Duration,
    next_allowed: Option<Instant>,
    offered: Option<Step>,
    waiting: Option<Step>,
}
impl Walking {
    fn acknowledge(
        &mut self,
        outcome: Outcome,
        a: &super::battle_payload::data::Actor,
        now: Instant,
    ) -> Result<()> {
        let sent = self
            .waiting
            .as_ref()
            .context("route has no submitted step")?;
        ensure!(
            outcome
                == Outcome::Operation(
                    super::battle_action::Rejection::None,
                    super::battle_operation::Disposition::Applied,
                    super::battle_operation::Reason::None
                ),
            "server refused route step"
        );
        ensure!(
            a.last_move_sequence == sent.sequence
                && a.heading == i32::from(sent.heading)
                && (a.position.x, a.position.y) == (sent.to.x, sent.to.y),
            "route receipt pose differs"
        );
        let index = self.index.checked_add(1).context("route index overflow")?;
        let next_allowed = now
            .checked_add(self.interval)
            .context("route clock overflow")?;
        self.sequence = sent.sequence;
        self.index = index;
        self.waiting = None;
        self.next_allowed = Some(next_allowed);
        Ok(())
    }
}
pub(crate) struct Route {
    desired: Option<Input>,
    job: Option<Job>,
    walking: Option<Walking>,
    status: Status,
}
impl Default for Route {
    fn default() -> Self {
        Self {
            desired: None,
            job: None,
            walking: None,
            status: Status::Idle,
        }
    }
}
impl Route {
    pub(crate) fn cancel(&mut self) {
        self.desired = None;
        self.walking = None;
        if let Some(job) = &self.job {
            job.cancel();
        }
        self.status = if self.job.is_some() {
            Status::Stopping
        } else {
            Status::Idle
        };
    }
    fn stop(&mut self, reason: &'static str) {
        self.cancel();
        self.status = Status::Stopped(reason);
    }
    pub(crate) fn is_quiescent(&self) -> bool {
        self.job.is_none()
    }
    pub(crate) fn status(&self) -> Status {
        self.status
    }
    pub(crate) fn poll(&mut self) {
        if self.job.as_ref().is_some_and(Job::finished) {
            let mut job = self.job.take().unwrap();
            let result = job.join();
            if !job.cancel.load(Ordering::Acquire) {
                match result {
                    Ok(w) => {
                        self.walking = Some(w);
                        self.status = Status::Walking;
                    }
                    Err(_) => self.stop("找不到可通行的路徑，請重新點選較近的位置。"),
                }
            }
        }
        if self.job.is_none() {
            if let Some(input) = self.desired.take() {
                match Job::start(input) {
                    Ok(job) => {
                        self.job = Some(job);
                        self.status = Status::Planning;
                    }
                    Err(_) => self.stop("無法啟動尋路工作；未送出移動。"),
                }
            } else if self.status == Status::Stopping {
                self.status = Status::Idle;
            }
        }
    }
    pub(crate) fn request(
        &mut self,
        surface: &Surface,
        intent: Intent,
        view: &LoginSnapshot,
    ) -> Result<()> {
        super::battle_input::validate(intent, view)?;
        ensure!(
            intent.stamp == Stamp::of(&surface.observation)
                && super::battle_player::controls_ready(surface, view),
            "route picture stale"
        );
        let Kind::Navigate(goal) = intent.kind else {
            anyhow::bail!("not a ground intent")
        };
        let navigation = surface
            .navigation
            .clone()
            .context("route geometry missing")?;
        self.cancel();
        self.desired = Some(Input {
            navigation,
            stamp: intent.stamp,
            goal,
        });
        self.poll();
        Ok(())
    }
    pub(crate) fn next(
        &mut self,
        surface: Option<&Surface>,
        view: &LoginSnapshot,
        now: Instant,
    ) -> Option<Command> {
        self.poll();
        let result = self.offer(surface, view, now);
        match result {
            Ok(q) => q,
            Err(_) => {
                self.stop("角色、路徑或伺服器回覆已更新；移動已停止，請重新點選。");
                None
            }
        }
    }
    fn offer(
        &mut self,
        surface: Option<&Surface>,
        view: &LoginSnapshot,
        now: Instant,
    ) -> Result<Option<Command>> {
        let Some(w) = self.walking.as_mut() else {
            return Ok(None);
        };
        super::battle_input::validate(
            Intent {
                stamp: w.stamp,
                kind: Kind::Navigate(*w.plan.cells().last().context("empty route")?),
            },
            view,
        )?;
        let o = view
            .canonical
            .observation()
            .context("route owner missing")?;
        let a = o
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == w.stamp.entity)
            .context("route actor missing")?;
        ensure!(
            w.interval.as_nanos() == a.step_interval_nanos as u128,
            "route pace changed"
        );
        if view.world_pending {
            return Ok(None);
        }
        if let Some(sent) = &w.waiting {
            let receipt = view
                .canonical
                .received_after(&sent.command, sent.prior)
                .context("route did not receive exact command")?;
            w.acknowledge(receipt, a, now)?;
        }
        ensure!(
            w.sequence == a.last_move_sequence
                && w.plan.cells().get(w.index)
                    == Some(&WorldCell {
                        x: a.position.x,
                        y: a.position.y
                    }),
            "route origin changed"
        );
        if w.index == w.plan.steps() {
            self.walking = None;
            self.status = Status::Arrived;
            return Ok(None);
        }
        if w.next_allowed.is_some_and(|t| now < t) {
            return Ok(None);
        }
        let Some(surface) = surface.filter(|s| super::battle_player::controls_ready(s, view))
        else {
            return Ok(None);
        };
        let heading = *w.plan.headings().get(w.index).context("route heading")?;
        let intent = Intent {
            stamp: Stamp::of(&surface.observation),
            kind: Kind::Local(Local::Step(heading)),
        };
        let command = super::battle_input::command(surface, intent, view)?;
        w.offered = Some(Step {
            command: command.clone(),
            prior: view.canonical.receipt_epoch(),
            to: w.plan.cells()[w.index + 1],
            sequence: w
                .sequence
                .checked_add(1)
                .context("route sequence overflow")?,
            heading,
        });
        self.status = Status::Walking;
        Ok(Some(command))
    }
    pub(crate) fn submitted(&mut self, command: &Command) -> Result<()> {
        let w = self.walking.as_mut().context("route missing")?;
        ensure!(
            w.waiting.is_none() && w.offered.as_ref().is_some_and(|s| s.command == *command),
            "route command changed before admission"
        );
        w.waiting = w.offered.take();
        self.status = Status::WaitingForAck;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn offered() -> (Walking, super::super::battle_payload::data::Actor) {
        let (map, mut raw) = super::super::battle_space::tests::sample();
        raw.world.phase = super::super::battle_payload::data::Phase::Active;
        raw.world.revision = 2;
        raw.world.active_elapsed_nanos = 0;
        raw.world.loading_elapsed_nanos = 1;
        for a in &mut raw.actors.0 {
            a.loaded = true;
        }
        let o = super::super::battle_scene::tests::observation(raw);
        let stamp = Stamp::of(&o);
        let actor = o
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == stamp.entity)
            .unwrap()
            .clone();
        let space =
            super::super::battle_space::Space::qualify(map, o.clone(), &AtomicBool::new(false))
                .unwrap();
        let navigation = space.snapshot(o, &AtomicBool::new(false)).unwrap();
        let mut w = Job::start(Input {
            navigation,
            stamp,
            goal: WorldCell { x: 32770, y: 32770 },
        })
        .unwrap()
        .join()
        .unwrap();
        assert!(w.plan.steps() > 0);
        w.offered = Some(Step {
            command: Command::world(super::super::battle_world::Request::read(
                stamp.context,
                stamp.channel,
                None,
            )),
            prior: Some(5),
            to: w.plan.cells()[1],
            sequence: w.sequence + 1,
            heading: w.plan.headings()[0],
        });
        (w, actor)
    }
    fn applied() -> Outcome {
        Outcome::Operation(
            super::super::battle_action::Rejection::None,
            super::super::battle_operation::Disposition::Applied,
            super::super::battle_operation::Reason::None,
        )
    }
    #[test]
    fn offered_step_is_not_an_ack_and_cancel_removes_unsent_and_waiting_work() {
        let (w, actor) = offered();
        let command = w.offered.as_ref().unwrap().command.clone();
        let mut route = Route {
            walking: Some(w),
            ..Default::default()
        };
        assert!(route
            .walking
            .as_mut()
            .unwrap()
            .acknowledge(applied(), &actor, Instant::now())
            .is_err());
        let wrong = Command::world(super::super::battle_world::Request::read(
            route.walking.as_ref().unwrap().stamp.context,
            route.walking.as_ref().unwrap().stamp.channel ^ 1,
            None,
        ));
        assert!(route.submitted(&wrong).is_err());
        route.submitted(&command).unwrap();
        assert_eq!(route.walking.as_ref().unwrap().index, 0);
        assert_eq!(route.status(), Status::WaitingForAck);
        assert!(route.submitted(&command).is_err());
        route.cancel();
        assert!(route.walking.is_none() && route.desired.is_none() && route.is_quiescent());
        assert!(route.submitted(&command).is_err());
    }
    #[test]
    fn refused_or_mismatched_pose_cannot_advance_or_start_next_step_clock() {
        let (mut w, mut actor) = offered();
        w.waiting = w.offered.take();
        let sent = w.waiting.as_ref().unwrap();
        actor.position.x = sent.to.x;
        actor.position.y = sent.to.y;
        actor.heading = i32::from(sent.heading);
        actor.last_move_sequence = sent.sequence;
        let sequence = w.sequence;
        for field in 0..5 {
            let mut bad = actor.clone();
            match field {
                0 => bad.position.x += 1,
                1 => bad.position.y += 1,
                2 => bad.heading ^= 1,
                3 => bad.last_move_sequence -= 1,
                _ => bad.last_move_sequence += 1,
            }
            assert!(w.acknowledge(applied(), &bad, Instant::now()).is_err());
            assert_eq!((w.index, w.sequence), (0, sequence));
            assert!(w.waiting.is_some() && w.next_allowed.is_none());
        }
        let denied = Outcome::Operation(
            super::super::battle_action::Rejection::None,
            super::super::battle_operation::Disposition::Denied,
            super::super::battle_operation::Reason::Blocked,
        );
        assert!(w.acknowledge(denied, &actor, Instant::now()).is_err());
        assert_eq!((w.index, w.sequence), (0, sequence));
        assert!(w.waiting.is_some() && w.next_allowed.is_none());
        let now = Instant::now();
        w.acknowledge(applied(), &actor, now).unwrap();
        assert_eq!((w.index, w.sequence), (1, actor.last_move_sequence));
        assert_eq!(w.next_allowed, Some(now + w.interval));
        assert!(w.waiting.is_none());
        assert!(w.acknowledge(applied(), &actor, now).is_err());
        assert_eq!(w.index, 1);
    }
    #[test]
    fn bounded_worker_cancels_and_joins_without_any_detached_task() {
        let (map, mut raw) = super::super::battle_space::tests::sample();
        raw.world.phase = super::super::battle_payload::data::Phase::Active;
        raw.world.revision = 2;
        raw.world.active_elapsed_nanos = 0;
        raw.world.loading_elapsed_nanos = 1;
        for a in &mut raw.actors.0 {
            a.loaded = true;
        }
        let o = super::super::battle_scene::tests::observation(raw);
        let space =
            super::super::battle_space::Space::qualify(map, o.clone(), &AtomicBool::new(false))
                .unwrap();
        let n = space.snapshot(o.clone(), &AtomicBool::new(false)).unwrap();
        let mut route = Route::default();
        route.desired = Some(Input {
            navigation: n,
            stamp: Stamp::of(&o),
            goal: WorldCell { x: 32800, y: 32800 },
        });
        route.poll();
        assert!(!route.is_quiescent());
        route.cancel();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !route.is_quiescent() {
            route.poll();
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert!(route.walking.is_none() && route.desired.is_none());
    }
}
