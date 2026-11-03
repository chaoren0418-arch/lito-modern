//! Focus-independent presentation timeline for one idle loop and one action.
//!
//! This is not the authority for movement, attacks, damage or game state.
//! Looping is allowed only for an event-free idle animation. Action sound cues
//! that have expired are counted, never burst-played after a paused UI thread.

use super::{animation::AnimationClock, sprite_actions::FrameEvent};
use anyhow::{ensure, Context, Result};
use std::time::Duration;

const NANOS: u128 = 1_000_000_000;
const MAX_ACTION_EVENTS: usize = 64;
const MAX_SOUNDS_PER_UPDATE: usize = 4;
const SOUND_MAX_AGE: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActorPhase {
    Idle,
    Action,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PresentationFrame {
    pub phase: ActorPhase,
    pub source_frame: usize,
    pub idle_cycle: u128,
    pub sounds: Vec<u32>,
    pub hit_cues: usize,
    pub expired_sounds: usize,
}

pub struct ActorTimeline {
    idle: AnimationClock,
    action: AnimationClock,
    idle_origin: Duration,
    idle_offset_ticks: u64,
    idle_offset_fps: u32,
    idle_cycle: Option<u128>,
    action_start: Option<Duration>,
    last_now: Option<Duration>,
}

impl ActorTimeline {
    pub fn new(idle: AnimationClock, action: AnimationClock) -> Result<Self> {
        ensure!(
            idle.animation().frames.iter().all(|f| f.events.is_empty()),
            "idle loops containing events require a separately verified event policy"
        );
        let mut events = 0usize;
        for frame in &action.animation().frames {
            events = events
                .checked_add(frame.events.len())
                .context("event count overflow")?;
            ensure!(
                events <= MAX_ACTION_EVENTS,
                "presentation action event budget exceeded"
            );
            ensure!(
                frame
                    .events
                    .iter()
                    .all(|e| matches!(e, FrameEvent::Sound(_) | FrameEvent::Hit)),
                "action requires an unimplemented overlay/effect event renderer"
            );
        }
        Ok(Self {
            idle,
            action,
            idle_origin: Duration::ZERO,
            idle_offset_ticks: 0,
            idle_offset_fps: 1,
            idle_cycle: None,
            action_start: None,
            last_now: None,
        })
    }

    /// Starts or explicitly restarts the inspected action; key auto-repeat is
    /// filtered by the window input owner. It sends no gameplay command.
    pub fn trigger_action(&mut self, now: Duration) -> Result<()> {
        self.check_time(now)?;
        self.action.restart();
        self.action_start = Some(now);
        self.last_now = Some(now);
        Ok(())
    }

    fn check_time(&self, now: Duration) -> Result<()> {
        ensure!(
            self.last_now.is_none_or(|last| now >= last),
            "presentation time went backwards"
        );
        Ok(())
    }

    pub fn step(&mut self, now: Duration) -> Result<PresentationFrame> {
        self.check_time(now)?;
        let mut sounds = Vec::with_capacity(MAX_SOUNDS_PER_UPDATE);
        let mut hit_cues = 0;
        let mut expired_sounds = 0;
        if let Some(start) = self.action_start {
            let elapsed = now.checked_sub(start).context("action time underflow")?;
            let fps = self.action.animation().frames_per_second;
            let action_ticks = self.action.total_ticks();
            let sample = self.action.sample(elapsed)?;
            for cue in &sample.events {
                match *cue.event {
                    FrameEvent::Sound(id) => {
                        let deadline = tick_time(cue.tick, fps)?;
                        if elapsed.saturating_sub(deadline) <= SOUND_MAX_AGE
                            && sounds.len() < MAX_SOUNDS_PER_UPDATE
                        {
                            sounds.push(id);
                        } else {
                            expired_sounds += 1;
                        }
                    }
                    FrameEvent::Hit => hit_cues += 1,
                    _ => unreachable!("validated by ActorTimeline::new"),
                }
            }
            if !sample.finished {
                let source_frame = sample.frame_index;
                self.last_now = Some(now);
                return Ok(PresentationFrame {
                    phase: ActorPhase::Action,
                    source_frame,
                    idle_cycle: 0,
                    sounds,
                    hit_cues,
                    expired_sounds,
                });
            }
            self.action_start = None;
            // Keep the exact rational end. Rounding the new origin to a
            // nanosecond loses a source tick at later exact frame boundaries.
            self.idle_origin = start;
            self.idle_offset_ticks = action_ticks;
            self.idle_offset_fps = fps;
            self.idle_cycle = None;
        }
        let elapsed = now
            .checked_sub(self.idle_origin)
            .context("idle time underflow")?;
        let fps = self.idle.animation().frames_per_second;
        // Divide cumulative rational SOURCE ticks, not rounded frame or cycle
        // durations. A large delay jumps directly to its cycle in bounded work.
        let elapsed_scaled = elapsed
            .as_nanos()
            .checked_mul(u128::from(self.idle_offset_fps))
            .context("idle origin scale overflow")?;
        let after_origin = elapsed_scaled
            .checked_sub(u128::from(self.idle_offset_ticks) * NANOS)
            .context("idle time before exact action end")?;
        let absolute_tick = after_origin
            .checked_mul(u128::from(fps))
            .context("idle time overflow")?
            / (NANOS * u128::from(self.idle_offset_fps));
        let total_ticks = u128::from(self.idle.total_ticks());
        let cycle = absolute_tick / total_ticks;
        let local_tick = (absolute_tick % total_ticks) as u64;
        if self.idle_cycle != Some(cycle) {
            self.idle.restart();
            self.idle_cycle = Some(cycle);
        }
        let source_frame = self.idle.sample(tick_time(local_tick, fps)?)?.frame_index;
        self.last_now = Some(now);
        Ok(PresentationFrame {
            phase: ActorPhase::Idle,
            source_frame,
            idle_cycle: cycle,
            sounds,
            hit_cues,
            expired_sounds,
        })
    }
}

fn tick_time(tick: u64, fps: u32) -> Result<Duration> {
    ensure!(fps > 0, "invalid frame rate");
    let nanos = (u128::from(tick) * NANOS).div_ceil(u128::from(fps));
    Ok(Duration::new(
        (nanos / NANOS).try_into()?,
        (nanos % NANOS) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::sprite_actions::{ActionFrame, Animation, Directionality};
    fn clock(ticks: &[u32], events: &[(usize, FrameEvent)]) -> AnimationClock {
        let mut frames: Vec<_> = ticks
            .iter()
            .map(|ticks| ActionFrame {
                image_id: 0,
                subframe: 0,
                duration_ticks: *ticks,
                events: vec![],
            })
            .collect();
        for (i, event) in events {
            frames[*i].events.push(event.clone());
        }
        AnimationClock::new(&Animation {
            label_bytes: vec![],
            directionality: Directionality::Undirected,
            frames_per_second: 24,
            frames,
        })
        .unwrap()
    }
    fn timeline() -> ActorTimeline {
        ActorTimeline::new(
            clock(&[1, 1], &[]),
            clock(
                &[0, 3, 3, 3, 5, 2, 4],
                &[(4, FrameEvent::Sound(1539)), (5, FrameEvent::Hit)],
            ),
        )
        .unwrap()
    }
    #[test]
    fn idle_is_rational_and_focus_independent_without_catchup_loops() {
        let mut player = timeline();
        assert_eq!(player.step(Duration::ZERO).unwrap().source_frame, 0);
        assert_eq!(
            player
                .step(Duration::from_nanos(41_666_667))
                .unwrap()
                .source_frame,
            1
        );
        let one_second = player.step(Duration::from_secs(1)).unwrap();
        assert_eq!((one_second.idle_cycle, one_second.source_frame), (12, 0));
        let jump = player.step(Duration::from_secs(1_000_000)).unwrap();
        assert_eq!((jump.idle_cycle, jump.source_frame), (12_000_000, 0));
        assert!(jump.sounds.is_empty());
    }
    #[test]
    fn attack_cues_once_then_returns_to_idle_with_remainder_time() {
        let mut player = timeline();
        player.trigger_action(Duration::from_secs(2)).unwrap();
        assert_eq!(
            player.step(Duration::from_secs(2)).unwrap().phase,
            ActorPhase::Action
        );
        let sound = player.step(Duration::from_millis(2375)).unwrap();
        assert_eq!(sound.sounds, [1539]);
        assert!(player
            .step(Duration::from_millis(2375))
            .unwrap()
            .sounds
            .is_empty());
        assert_eq!(
            player.step(Duration::from_millis(2600)).unwrap().hit_cues,
            1
        );
        let idle = player.step(Duration::from_secs(3)).unwrap();
        assert_eq!(idle.phase, ActorPhase::Idle);
        assert_eq!(idle.hit_cues, 0);
        player.trigger_action(Duration::from_secs(3)).unwrap();
        assert_eq!(
            player.step(Duration::from_millis(3375)).unwrap().sounds,
            [1539]
        );
    }
    #[test]
    fn late_sound_is_counted_not_burst_played_and_backwards_time_rejected() {
        let mut player = timeline();
        player.trigger_action(Duration::ZERO).unwrap();
        let jump = player.step(Duration::from_secs(60)).unwrap();
        assert_eq!(
            (jump.phase, jump.expired_sounds, jump.hit_cues),
            (ActorPhase::Idle, 1, 1)
        );
        assert!(jump.sounds.is_empty());
        assert!(player.step(Duration::ZERO).is_err());
        assert!(player.trigger_action(Duration::ZERO).is_err());
        assert_eq!(player.step(Duration::from_secs(60)).unwrap().hit_cues, 0);
    }
    #[test]
    fn action_to_idle_keeps_fractional_tick_phase_at_exact_boundaries() {
        let mut player = timeline();
        player.trigger_action(Duration::ZERO).unwrap();
        let frame = player.step(Duration::from_secs(1)).unwrap();
        assert_eq!(
            (frame.phase, frame.idle_cycle, frame.source_frame),
            (ActorPhase::Idle, 2, 0)
        );
    }
    #[test]
    fn rejects_unimplemented_presentation_semantics_before_playback() {
        assert!(
            ActorTimeline::new(clock(&[1], &[(0, FrameEvent::Sound(1))]), clock(&[1], &[]))
                .is_err()
        );
        assert!(
            ActorTimeline::new(clock(&[1], &[]), clock(&[1], &[(0, FrameEvent::Effect(8))]))
                .is_err()
        );
    }
}
