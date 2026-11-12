//! Player movement callbacks sharing the 4D6570 queue with static 2941.
//! Input owns held/cancel; successful sends own action_sent. Neither a pose
//! sampled by a worker nor a pending server acknowledgement creates these bits.
use super::{
    native_navigation::InputStamp,
    native_static2941::{CallbackHost, Queue},
    native_world::{Actor, World},
    native_world_animation::{resolve, Pose},
    sprite_actions::{Animation, Directionality, SpriteActionCatalog},
};
use anyhow::{ensure, Context, Result};
use std::sync::Arc;
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Player,
    Camera,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
    Player(u64),
    Camera,
}
impl Event {
    pub(crate) fn kind(self) -> Kind {
        match self {
            Self::Player(_) => Kind::Player,
            Self::Camera => Kind::Camera,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WalkPlan {
    actor_graphics: u16,
    graphics: u32,
    action: u8,
    animation: Animation,
}
impl WalkPlan {
    pub(crate) fn frame_count(&self) -> usize {
        self.animation.frames.len()
    }
    pub(crate) fn load(catalog: &SpriteActionCatalog, actor: &Actor) -> Result<Self> {
        let clip = resolve(catalog, u32::from(actor.graphics), actor.walk_action)?;
        ensure!(
            matches!(clip.animation.frames.len(), 4 | 6 | 8 | 12),
            "player source walk frame count"
        );
        ensure!(
            clip.animation.directionality == Directionality::EightWay,
            "player walk directions"
        );
        // This producer does not synthesize an event interpreter. Ordinary
        // movement sound events continue to be consumed by the renderer.
        Ok(Self {
            actor_graphics: actor.graphics,
            graphics: clip.graphics,
            action: actor.walk_action,
            animation: clip.animation,
        })
    }
    fn duration(&self, frame: usize, actor: &Actor, special: bool) -> Result<u32> {
        let frame = self
            .animation
            .frames
            .get(frame)
            .context("player source frame")?;
        let mut ms = super::native_actor_inspection::catalog_frame_ms(
            u32::from(frame.duration_ticks),
            self.animation.frames_per_second,
        )?;
        // 4E8750 truncates after EACH multiply/divide, not one combined ratio.
        if special {
            ms = ms * 7 / 16;
        } else {
            ms = match actor.speed {
                0 => ms,
                1 => ms * 3 / 4,
                2 => ms * 2,
                17 => ms * 3 / 4,
                18 => ms * 2,
                _ => anyhow::bail!("player speed mode"),
            };
        }
        ms = match actor.brave {
            0 | 6 => ms,
            1 | 4 => ms * 3 / 4,
            3 => ms * 7 / 8,
            5 => ms * 3 / 8,
            _ => anyhow::bail!("player brave mode"),
        };
        Ok(ms)
    }
}

/// A source-index pose, copied into World snapshots. The binding prevents a
/// completed walk from replacing a new combat action or a corrected position.
#[derive(Clone, Debug)]
pub(crate) struct WalkPose {
    stamp: InputStamp,
    motion: u64,
    action_sequence: u64,
    plan: Arc<WalkPlan>,
    pub(crate) frame: usize,
    pub(crate) complete: bool,
}
impl WalkPose {
    pub(crate) fn current(&self, world: &World) -> bool {
        self.stamp == InputStamp::capture(0, world)
            && world.own_actor().is_some_and(|a| {
                a.motion_revision == self.motion
                    && a.graphics == self.plan.actor_graphics
                    && a.walk_action == self.plan.action
            })
    }
    fn queued_actions(&self, world: &World) -> usize {
        world.own_actor().map_or(0, |a| {
            a.actions
                .iter()
                .filter(|cue| cue.sequence > self.action_sequence)
                .count()
        })
    }
    pub(crate) fn pose(&self, world: &World, actor: &Actor) -> Option<Result<Pose>> {
        if !self.current(world) || world.own_actor()?.id != actor.id {
            return None;
        }
        if self.complete {
            return None;
        }
        Some((|| {
            let frame = self
                .plan
                .animation
                .frames
                .get(self.frame)
                .context("player pose source index")?;
            let from = actor.motion_from.context("player walk origin")?;
            let ratio = if self.frame == 0 {
                1.
            } else {
                self.frame as f64 / self.plan.animation.frames.len() as f64
            };
            Ok(Pose {
                graphics: self.plan.graphics,
                image: frame.image_id + u32::from(actor.heading),
                subframe: frame.subframe,
                source_frame: self.frame,
                action: self.plan.action,
                phase: if self.complete { "idle" } else { "walk" },
                action_complete: self.complete,
                position: [
                    f64::from(from[0]) + (f64::from(actor.x) - f64::from(from[0])) * ratio,
                    f64::from(from[1]) + (f64::from(actor.y) - f64::from(from[1])) * ratio,
                ],
                events: frame.events.iter().map(|e| format!("{e:?}")).collect(),
                entered_events: Vec::new(),
                static_prop: false,
            })
        })())
    }
}

#[derive(Default)]
pub(crate) struct Control {
    stamp: Option<InputStamp>,
    serial: u64,
    held: Option<u8>,
    pending: Option<Zeroizing<Vec<u8>>>,
    deferred_sent: Option<std::time::Instant>,
    cancel: bool,
    active: bool,
    continuation: bool,
    pub(crate) action_sent: bool,
    pump: bool,
    pub(crate) pump_completed: bool,
    pub(crate) camera_step: i32,
    pub(crate) camera_actor: Option<u32>,
    previous_draw: Option<u32>,
    last_draw_frame: Option<u64>,
    pub(crate) raw_draw_delta: u32,
    pub(crate) capped_draw_delta: u32,
    skip_suffix_draw: bool,
    dirty: bool,
    // Selected-character mode is set by the controller, not by the renderer.
    pub(crate) special_speed: bool,
    special_speed_disabled: bool,
}
impl Control {
    pub(crate) fn synchronize(&mut self, q: &mut Queue, w: &mut World, c: u64) {
        q.synchronize(w, c);
        let stamp =
            (c != 0 && w.view_ready() && !w.observer_mode).then(|| InputStamp::capture(c, w));
        if self.stamp != stamp {
            q.cancel_native(Kind::Player);
            q.cancel_native(Kind::Camera);
            let serial = self.serial.wrapping_add(1);
            *self = Self {
                stamp,
                serial,
                camera_step: -1,
                ..Self::default()
            };
            w.player_walk = None;
        }
        if w.player_walk.as_ref().is_some_and(|p| !p.current(w)) {
            q.cancel_native(Kind::Player);
            w.player_walk = None;
            self.active = false;
            self.continuation = false;
            self.pending = None;
        }
    }
    pub(crate) fn held(&mut self, heading: Option<u8>) {
        self.held = heading.filter(|h| *h < 8);
    }
    pub(crate) fn character_timing(&mut self, mode: u8, level: u8) {
        // 492E6B reads directory +3E (reserved/mode), +37 (level).
        self.special_speed = mode != 0 && level <= 50;
        self.special_speed_disabled = mode == 7;
    }
    fn special(&self, actor: &Actor) -> bool {
        (self.special_speed || actor.status & 0x80 != 0) && !self.special_speed_disabled
    }
    pub(crate) fn cancel(&mut self) {
        self.cancel = true;
        self.held = None;
    }
    pub(crate) fn moving(&self) -> bool {
        self.active
    }
    pub(crate) fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub(crate) fn cancel_pending(&mut self) {
        self.pending = None;
    }
    /// Offline fixtures only: install a frame-zero deferred body without a walk.
    #[cfg(test)]
    pub(crate) fn defer_for_test(&mut self, body: Zeroizing<Vec<u8>>) {
        self.pending = Some(body);
    }
    pub(crate) fn take_deferred_sent(&mut self) -> Option<std::time::Instant> {
        self.deferred_sent.take()
    }
    /// Called only after request validation. Sending is delayed until the
    /// frame-zero suffix when an owned source walk is still in progress.
    pub(crate) fn defer(&mut self, body: Zeroizing<Vec<u8>>) -> Result<Option<Zeroizing<Vec<u8>>>> {
        if !self.active {
            return Ok(Some(body));
        }
        ensure!(self.pending.is_none(), "player action already queued");
        self.pending = Some(body);
        self.held = None;
        Ok(None)
    }
    pub(crate) fn sent(&mut self) {
        self.action_sent = true;
    }
    pub(crate) fn accepted(&mut self, body: &[u8], w: &World) {
        // S37 has already passed World's exact envelope/actor validation.
        if body.len() == 7
            && body[0] == 0x37
            && self.action_sent
            && u16::from_le_bytes([body[5], body[6]]) == 169
            && w.own_actor()
                .is_some_and(|a| a.id == u32::from_le_bytes(body[1..5].try_into().unwrap()))
        {
            self.pump = true;
        }
        // Source 4A4670 clears the command flag for message 0x49.
        if body.len() >= 4 && body[0] == 0x10 && u16::from_le_bytes([body[1], body[2]]) == 0x49 {
            self.action_sent = false;
        }
    }
    pub(crate) fn start(
        &mut self,
        q: &mut Queue,
        w: &mut World,
        c: u64,
        plan: Arc<WalkPlan>,
        clock: &mut impl FnMut() -> Result<u32>,
    ) -> Result<()> {
        self.synchronize(q, w, c);
        let actor = w.own_actor().context("player callback owner")?;
        ensure!(
            plan.actor_graphics == actor.graphics
                && plan.action == actor.walk_action
                && actor.motion_from.is_some(),
            "player callback plan changed"
        );
        self.serial = self
            .serial
            .checked_add(1)
            .context("player callback serial overflow")?;
        q.cancel_native(Kind::Player);
        let special = self.special(actor);
        let durations: Vec<_> = (1..plan.animation.frames.len())
            .map(|f| plan.duration(f, actor, special))
            .collect::<Result<_>>()?;
        w.player_walk = Some(WalkPose {
            stamp: InputStamp::capture(0, w),
            motion: actor.motion_revision,
            action_sequence: actor.action_sequence,
            plan,
            frame: 1,
            complete: false,
        });
        self.active = true;
        self.continuation = false;
        self.cancel = false;
        self.dirty = true;
        let mut delay = 0u32;
        for duration in durations {
            delay = delay.wrapping_add(duration);
            q.enqueue_native(w, c, delay, clock()?, Event::Player(self.serial))?;
        }
        w.visual_revision = w
            .visual_revision
            .checked_add(1)
            .context("player visual revision")?;
        Ok(())
    }
    fn normal_tail(&mut self, sample: u32, base_count: usize) {
        let delta = self.previous_draw.map_or(0, |old| sample.wrapping_sub(old));
        self.previous_draw = Some(sample);
        self.raw_draw_delta = delta;
        self.skip_suffix_draw = base_count == 12 && delta > 40;
        self.capped_draw_delta = delta.min(500);
    }
    pub(crate) fn observe_frame(
        &mut self,
        id: u64,
        count: usize,
        clock: impl FnOnce() -> Result<u32>,
    ) -> Result<()> {
        if self.last_draw_frame != Some(id) {
            self.normal_tail(clock()?, count);
            self.last_draw_frame = Some(id);
        }
        Ok(())
    }
}

pub(crate) trait Driver {
    /// Must finish composition and readiness enqueue before returning. It may
    /// legally reenter the queue through the supplied host at a higher layer.
    fn draw(
        &mut self,
        q: &mut Queue,
        w: &mut World,
        c: u64,
        clock: &mut dyn FnMut() -> Result<u32>,
    ) -> Result<usize>;
    fn send(&mut self, connection: u64, body: Zeroizing<Vec<u8>>) -> Result<()>;
    fn validate_step(&self, world: &World, heading: u8) -> Result<()>;
    fn frame_id(&self) -> Option<u64> {
        None
    }
}
pub(crate) struct Host<'a, D, C> {
    pub control: &'a mut Control,
    pub driver: &'a mut D,
    pub clock: &'a mut C,
}
impl<D: Driver, C: FnMut() -> Result<u32>> Host<'_, D, C> {
    pub(crate) fn run(
        &mut self,
        q: &mut Queue,
        w: &mut World,
        c: u64,
        pump_only: bool,
    ) -> Result<()> {
        self.control.synchronize(q, w, c);
        if self.control.pump {
            self.control.pump = false;
            q.cancel_native(Kind::Camera);
            self.control.camera_step = -1;
            self.control.camera_actor = None;
            self.control.pump_completed = false;
            if q.drain_4857b0_with_host(w, c, self)? {
                self.control.pump_completed = true;
            }
        }
        if pump_only {
            return Ok(());
        }
        if self.control.dirty {
            self.control.dirty = false;
            self.suffix(q, w, c)?;
        }
        let cutoff = self.sample()?;
        q.drain_with(w, c, cutoff, self)?;
        Ok(())
    }
    fn suffix(&mut self, q: &mut Queue, w: &mut World, c: u64) -> Result<()> {
        let Some(pose) = &w.player_walk else {
            return Ok(());
        };
        if pose.frame != 0 {
            if !self.control.skip_suffix_draw {
                self.draw(q, w, c)?;
            }
            return Ok(());
        }
        // 4E3207: cancel takes precedence over pending, then EB3.
        if self.control.cancel {
            q.cancel_native(Kind::Player);
            self.control.pending = None;
            self.control.continuation = false;
            self.control.active = false;
            self.control.cancel = false;
        } else if let Some(body) = self.control.pending.take() {
            self.driver.send(c, body)?;
            self.control.sent();
            self.control.deferred_sent = Some(std::time::Instant::now());
            self.control.active = false;
        } else if self.control.continuation {
            if !self.control.skip_suffix_draw {
                self.draw(q, w, c)?;
            }
        } else {
            self.control.active = false;
        }
        if !self.control.active {
            if let Some(p) = &mut w.player_walk {
                p.complete = true;
            }
        }
        Ok(())
    }
}
impl<D: Driver, C: FnMut() -> Result<u32>> CallbackHost for Host<'_, D, C> {
    fn sample(&mut self) -> Result<u32> {
        (self.clock)()
    }
    fn draw(&mut self, q: &mut Queue, w: &mut World, c: u64) -> Result<()> {
        let scope = q.scope_token();
        let stamp = InputStamp::capture(c, w);
        let count = self.driver.draw(q, w, c, self.clock)?;
        if !q.same_scope(&scope) || stamp != InputStamp::capture(c, w) || !w.view_ready() {
            return Ok(());
        }
        let sample = self.sample()?;
        self.control.normal_tail(sample, count);
        self.control.last_draw_frame = self.driver.frame_id();
        Ok(())
    }
    fn dispatch(&mut self, q: &mut Queue, w: &mut World, c: u64, event: Event) -> Result<()> {
        self.control.synchronize(q, w, c);
        match event {
            Event::Camera => {
                q.cancel_native(Kind::Camera);
                if self.control.camera_step == 29 {
                    self.control.camera_step = -1;
                    self.control.camera_actor = None;
                } else {
                    self.control.camera_step += 1;
                    q.enqueue_native(w, c, 100, self.sample()?, Event::Camera)?;
                }
            }
            Event::Player(serial) => {
                if serial != self.control.serial {
                    return Ok(());
                }
                let Some(pose) = w.player_walk.as_ref() else {
                    return Ok(());
                };
                self.control.continuation = false; // entry 4E21F9
                if pose.frame == 0 {
                    let plan = pose.plan.clone();
                    w.visual_revision = w
                        .visual_revision
                        .checked_add(1)
                        .context("player arrival revision")?;
                    if self.control.pending.is_none() && !self.control.cancel {
                        if let Some(heading) = self.control.held {
                            if self.driver.validate_step(w, heading).is_ok() {
                                let body = w.move_request(heading)?;
                                self.driver.send(c, Zeroizing::new(body))?;
                                w.predict_step(heading)?;
                                self.control.start(q, w, c, plan, self.clock)?;
                                self.control.dirty = false;
                                return self.suffix(q, w, c);
                            }
                        }
                        self.control.active = false;
                        w.player_walk.as_mut().unwrap().complete = true;
                    }
                    return self.suffix(q, w, c);
                }
                let p = w.player_walk.as_mut().unwrap();
                p.frame = (p.frame + 1) % p.plan.animation.frames.len();
                let ended = p.frame == 0;
                let plan = p.plan.clone();
                w.visual_revision = w
                    .visual_revision
                    .checked_add(1)
                    .context("player visual revision")?;
                if ended
                    && self.control.held.is_some()
                    && w.player_walk.as_ref().unwrap().queued_actions(w) == 0
                {
                    let a = w.own_actor().context("player end owner")?;
                    let duration = plan.duration(0, a, self.control.special(a))?;
                    self.control.continuation = true;
                    q.enqueue_native(w, c, duration, self.sample()?, Event::Player(serial))?;
                }
                self.suffix(q, w, c)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "native_player_callback_tests.rs"]
mod tests;
