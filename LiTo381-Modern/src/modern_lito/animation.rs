//! Single-playback clock for verified shared `list.spr` animations.
//!
//! Time is supplied by the caller, independently of focus, paint frequency or
//! a window thread. Boundaries use cumulative source ticks / source FPS, not
//! rounded per-frame milliseconds. Events are presentation cues only: a Hit
//! cue is never authority to damage an entity or send a combat result.

use std::{fmt, time::Duration};

use super::sprite_actions::{ActionFrame, Animation, FrameEvent};

const NANOS_PER_SECOND: u128 = 1_000_000_000;

#[derive(Clone, Copy, Debug)]
pub struct PlaybackLimits {
    pub max_frames: usize,
    /// Bounds both retained event metadata and worst-case delivery after one
    /// arbitrarily large time jump. There is no loop or unbounded catch-up.
    pub max_events: usize,
}

impl Default for PlaybackLimits {
    fn default() -> Self {
        Self {
            max_frames: 16_384,
            max_events: 65_536,
        }
    }
}

impl PlaybackLimits {
    fn validate(self) -> Result<Self, AnimationError> {
        if self.max_frames == 0 || self.max_frames > 65_536 || self.max_events > 262_144 {
            return Err(AnimationError::InvalidLimits);
        }
        Ok(self)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum AnimationError {
    InvalidLimits,
    EmptyAnimation,
    InvalidFrameRate(u32),
    ZeroDuration,
    LimitExceeded(&'static str),
    UnsupportedEvent {
        frame_index: usize,
        event_index: usize,
    },
    TimeWentBackwards {
        previous: Duration,
        requested: Duration,
    },
    ArithmeticOverflow,
    AllocationFailed,
}

impl fmt::Display for AnimationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits => write!(f, "invalid animation playback limits"),
            Self::EmptyAnimation => write!(f, "animation contains no frames"),
            Self::InvalidFrameRate(fps) => {
                write!(f, "animation FPS {fps} is outside verified 1..1000")
            }
            Self::ZeroDuration => write!(f, "animation has no positive-duration frame"),
            Self::LimitExceeded(what) => write!(f, "animation exceeds {what} budget"),
            Self::UnsupportedEvent {
                frame_index,
                event_index,
            } => write!(
                f,
                "uninterpreted animation event at frame {frame_index}, event {event_index}"
            ),
            Self::TimeWentBackwards {
                previous,
                requested,
            } => write!(
                f,
                "animation elapsed time went backwards from {previous:?} to {requested:?}"
            ),
            Self::ArithmeticOverflow => write!(f, "animation clock arithmetic overflow"),
            Self::AllocationFailed => write!(f, "unable to allocate bounded animation state"),
        }
    }
}
impl std::error::Error for AnimationError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentationEvent<'a> {
    pub frame_index: usize,
    pub event_index: usize,
    /// Exact cumulative source tick at which this frame is entered.
    pub tick: u64,
    /// An immutable reference preserves source event identity and ordering.
    pub event: &'a FrameEvent,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AnimationSample<'a> {
    /// Source action-frame index, not the SPR's subframe index.
    pub frame_index: usize,
    pub image_id: u32,
    pub subframe: u32,
    pub finished: bool,
    pub events: Vec<PresentationEvent<'a>>,
    /// Absolute elapsed deadline from playback start, rounded UP to one ns.
    /// At a late sample, this is the next actual boundary, never a stale one.
    /// None means playback is complete and no timer needs to be armed.
    pub next_deadline: Option<Duration>,
}

/// Owns one validated immutable animation snapshot, not the source catalog.
/// The caller retains responsibility
/// for resolving graphics identity, heading and decoded SPRs. Zero-tick setup
/// frames have events but no visible dwell; after completion the last frame
/// with positive duration remains selected. A new clock explicitly restarts.
pub struct AnimationClock {
    animation: Animation,
    starts: Vec<u64>,
    ends: Vec<u64>,
    event_prefix: Vec<usize>,
    total_ticks: u64,
    last_timed_frame: usize,
    next_event_frame: usize,
    last_elapsed: Option<Duration>,
}

fn bounded_vec<T>(capacity: usize) -> Result<Vec<T>, AnimationError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| AnimationError::AllocationFailed)?;
    Ok(output)
}

impl AnimationClock {
    pub fn new(animation: &Animation) -> Result<Self, AnimationError> {
        Self::with_limits(animation, PlaybackLimits::default())
    }

    pub fn with_limits(
        animation: &Animation,
        limits: PlaybackLimits,
    ) -> Result<Self, AnimationError> {
        let limits = limits.validate()?;
        if animation.frames.is_empty() {
            return Err(AnimationError::EmptyAnimation);
        }
        if animation.frames.len() > limits.max_frames {
            return Err(AnimationError::LimitExceeded("frames"));
        }
        if animation.label_bytes.len() > 256 * 1024 {
            return Err(AnimationError::LimitExceeded("label bytes"));
        }
        if !(1..=1000).contains(&animation.frames_per_second) {
            return Err(AnimationError::InvalidFrameRate(
                animation.frames_per_second,
            ));
        }
        let mut total_events = 0usize;
        let mut total_ticks = 0u64;
        let mut last_timed_frame = None;
        // Validate event semantics and counts before allocating the timeline.
        for (frame_index, frame) in animation.frames.iter().enumerate() {
            total_events = total_events
                .checked_add(frame.events.len())
                .ok_or(AnimationError::LimitExceeded("events"))?;
            if total_events > limits.max_events {
                return Err(AnimationError::LimitExceeded("events"));
            }
            for (event_index, event) in frame.events.iter().enumerate() {
                if matches!(
                    event,
                    FrameEvent::Skip | FrameEvent::UninterpretedNumeric { .. }
                ) {
                    return Err(AnimationError::UnsupportedEvent {
                        frame_index,
                        event_index,
                    });
                }
            }
            total_ticks = total_ticks
                .checked_add(u64::from(frame.duration_ticks))
                .ok_or(AnimationError::ArithmeticOverflow)?;
            if frame.duration_ticks > 0 {
                last_timed_frame = Some(frame_index);
            }
        }
        let last_timed_frame = last_timed_frame.ok_or(AnimationError::ZeroDuration)?;
        let mut starts = bounded_vec(animation.frames.len())?;
        let mut ends = bounded_vec(animation.frames.len())?;
        let mut event_prefix = bounded_vec(animation.frames.len() + 1)?;
        let mut tick = 0u64;
        let mut events = 0usize;
        event_prefix.push(0);
        for frame in &animation.frames {
            starts.push(tick);
            // Both sums were checked and bounded above in the validation pass.
            tick += u64::from(frame.duration_ticks);
            events += frame.events.len();
            ends.push(tick);
            event_prefix.push(events);
        }
        // Snapshot only after the complete source has passed limits/semantics.
        // The caller can now drop the multi-megabyte catalog without creating
        // self-referential UI state or retaining unrelated sprite definitions.
        let mut label_bytes = bounded_vec(animation.label_bytes.len())?;
        label_bytes.extend_from_slice(&animation.label_bytes);
        let mut frames = bounded_vec(animation.frames.len())?;
        for frame in &animation.frames {
            let mut events = bounded_vec(frame.events.len())?;
            events.extend_from_slice(&frame.events);
            frames.push(ActionFrame {
                image_id: frame.image_id,
                subframe: frame.subframe,
                duration_ticks: frame.duration_ticks,
                events,
            });
        }
        let snapshot = Animation {
            label_bytes,
            directionality: animation.directionality,
            frames_per_second: animation.frames_per_second,
            frames,
        };
        Ok(Self {
            animation: snapshot,
            starts,
            ends,
            event_prefix,
            total_ticks,
            last_timed_frame,
            next_event_frame: 0,
            last_elapsed: None,
        })
    }

    pub fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    pub fn animation(&self) -> &Animation {
        &self.animation
    }

    /// Explicitly starts a new presentation playback of the same immutable
    /// validated source, without cloning frames or allocating during input.
    pub fn restart(&mut self) {
        self.next_event_frame = 0;
        self.last_elapsed = None;
    }

    /// The first representable nanosecond at or after the exact rational end.
    pub fn duration(&self) -> Result<Duration, AnimationError> {
        tick_deadline(self.total_ticks, self.animation.frames_per_second)
    }

    pub fn sample(&mut self, elapsed: Duration) -> Result<AnimationSample<'_>, AnimationError> {
        if let Some(previous) = self.last_elapsed {
            if elapsed < previous {
                return Err(AnimationError::TimeWentBackwards {
                    previous,
                    requested: elapsed,
                });
            }
        }
        let scaled_nanos = elapsed
            .as_nanos()
            .checked_mul(u128::from(self.animation.frames_per_second))
            .ok_or(AnimationError::ArithmeticOverflow)?;
        // Saturate only the query position at the validated non-looping end.
        // Large Duration values never create work proportional to elapsed time.
        let tick = (scaled_nanos / NANOS_PER_SECOND).min(u128::from(self.total_ticks)) as u64;
        let finished = tick == self.total_ticks;
        let frame_index = if finished {
            self.last_timed_frame
        } else {
            self.ends.partition_point(|end| *end <= tick)
        };
        let pending_end = self.starts.partition_point(|start| *start <= tick);
        let pending_events =
            self.event_prefix[pending_end] - self.event_prefix[self.next_event_frame];
        let mut events = bounded_vec(pending_events)?;
        for frame in self.next_event_frame..pending_end {
            for (event_index, event) in self.animation.frames[frame].events.iter().enumerate() {
                events.push(PresentationEvent {
                    frame_index: frame,
                    event_index,
                    tick: self.starts[frame],
                    event,
                });
            }
        }
        let next_deadline = if finished {
            None
        } else {
            Some(tick_deadline(
                self.ends[frame_index],
                self.animation.frames_per_second,
            )?)
        };
        let frame = &self.animation.frames[frame_index];
        let sample = AnimationSample {
            frame_index,
            image_id: frame.image_id,
            subframe: frame.subframe,
            finished,
            events,
            next_deadline,
        };
        // Commit only after all fallible work succeeds; an error cannot consume
        // cues, partially advance the timeline, or prevent a valid retry.
        self.next_event_frame = pending_end;
        self.last_elapsed = Some(elapsed);
        Ok(sample)
    }
}

fn tick_deadline(tick: u64, fps: u32) -> Result<Duration, AnimationError> {
    let numerator = u128::from(tick)
        .checked_mul(NANOS_PER_SECOND)
        .ok_or(AnimationError::ArithmeticOverflow)?;
    let divisor = u128::from(fps);
    let nanoseconds = numerator
        .checked_add(divisor - 1)
        .ok_or(AnimationError::ArithmeticOverflow)?
        / divisor;
    let seconds = u64::try_from(nanoseconds / NANOS_PER_SECOND)
        .map_err(|_| AnimationError::ArithmeticOverflow)?;
    Ok(Duration::new(
        seconds,
        (nanoseconds % NANOS_PER_SECOND) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::sprite_actions::{ActionFrame, Directionality};
    use super::*;

    fn animation(fps: u32, ticks: &[u32]) -> Animation {
        Animation {
            label_bytes: b"clock fixture".to_vec(),
            directionality: Directionality::Undirected,
            frames_per_second: fps,
            frames: ticks
                .iter()
                .enumerate()
                .map(|(index, duration_ticks)| ActionFrame {
                    image_id: 80 + index as u32,
                    subframe: 10 + index as u32,
                    duration_ticks: *duration_ticks,
                    events: vec![FrameEvent::Sound(index as u32)],
                })
                .collect(),
        }
    }

    #[test]
    fn fps24_uses_cumulative_rational_deadlines_without_millisecond_drift() {
        let source = animation(24, &[1; 24]);
        let mut clock = AnimationClock::new(&source).unwrap();
        assert_eq!(clock.total_ticks(), 24);
        assert_eq!(clock.duration().unwrap(), Duration::from_secs(1));
        let start = clock.sample(Duration::ZERO).unwrap();
        assert_eq!(start.next_deadline, Some(Duration::from_nanos(41_666_667)));
        assert_eq!(
            clock
                .sample(Duration::from_nanos(41_666_666))
                .unwrap()
                .frame_index,
            0
        );
        let next = clock.sample(Duration::from_nanos(41_666_667)).unwrap();
        assert_eq!(next.frame_index, 1);
        assert_eq!(next.next_deadline, Some(Duration::from_nanos(83_333_334)));
        assert_eq!(
            clock
                .sample(Duration::from_millis(125))
                .unwrap()
                .frame_index,
            3
        );
        let near_end = clock.sample(Duration::from_nanos(999_999_999)).unwrap();
        assert_eq!(near_end.frame_index, 23);
        assert!(!near_end.finished);
        let end = clock.sample(Duration::from_secs(1)).unwrap();
        assert!(end.finished);
        assert_eq!(end.next_deadline, None);
    }

    #[test]
    fn setup_and_internal_zero_tick_chains_emit_in_order_without_dwell() {
        let mut source = animation(24, &[0, 0, 2, 0, 0, 1]);
        source.frames[0].events = vec![FrameEvent::Sound(7), FrameEvent::Overlay(8)];
        source.frames[3].events = vec![FrameEvent::Hit, FrameEvent::Effect(9)];
        let mut clock = AnimationClock::new(&source).unwrap();
        let first = clock.sample(Duration::ZERO).unwrap();
        assert_eq!(
            (first.frame_index, first.image_id, first.subframe),
            (2, 82, 12)
        );
        assert_eq!(
            first
                .events
                .iter()
                .map(|e| (e.frame_index, e.event_index, e.tick))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0), (0, 1, 0), (1, 0, 0), (2, 0, 0)]
        );
        assert_eq!(first.events[0].event, &FrameEvent::Sound(7));
        let boundary = clock.sample(Duration::from_nanos(83_333_334)).unwrap();
        assert_eq!(boundary.frame_index, 5);
        assert_eq!(
            boundary
                .events
                .iter()
                .map(|e| (e.frame_index, e.event_index, e.tick))
                .collect::<Vec<_>>(),
            vec![(3, 0, 2), (3, 1, 2), (4, 0, 2), (5, 0, 2)]
        );
        assert_eq!(boundary.events[0].event, &FrameEvent::Hit);
        assert_eq!(boundary.next_deadline, Some(Duration::from_millis(125)));
    }

    #[test]
    fn a_large_first_sample_delivers_every_crossed_event_once_then_stays_finished() {
        let source = animation(30, &[2, 0, 3, 1, 0, 0]);
        let mut clock = AnimationClock::new(&source).unwrap();
        let sample = clock.sample(Duration::MAX).unwrap();
        assert!(sample.finished);
        assert_eq!(sample.frame_index, 3);
        assert_eq!(sample.events.len(), 6);
        assert_eq!(
            sample.events.iter().map(|e| e.tick).collect::<Vec<_>>(),
            vec![0, 2, 2, 5, 6, 6]
        );
        let again = clock.sample(Duration::MAX).unwrap();
        assert!(again.events.is_empty());
        assert_eq!(again.next_deadline, None);
        assert_eq!(again.subframe, 13);
    }

    #[test]
    fn repeated_samples_and_multiple_crossed_frames_preserve_order_without_duplicates() {
        let source = animation(10, &[1, 1, 1, 1, 1]);
        let mut clock = AnimationClock::new(&source).unwrap();
        assert_eq!(clock.sample(Duration::ZERO).unwrap().events.len(), 1);
        assert!(clock.sample(Duration::ZERO).unwrap().events.is_empty());
        let crossed = clock.sample(Duration::from_millis(350)).unwrap();
        assert_eq!(crossed.frame_index, 3);
        assert_eq!(
            crossed
                .events
                .iter()
                .map(|e| e.frame_index)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(crossed.next_deadline, Some(Duration::from_millis(400)));
        assert!(clock
            .sample(Duration::from_millis(350))
            .unwrap()
            .events
            .is_empty());
        assert_eq!(
            clock
                .sample(Duration::from_secs(20))
                .unwrap()
                .events
                .iter()
                .map(|e| e.frame_index)
                .collect::<Vec<_>>(),
            vec![4]
        );
    }

    #[test]
    fn backwards_time_fails_without_consuming_future_events_even_after_completion() {
        let source = animation(10, &[1, 1, 1]);
        let mut clock = AnimationClock::new(&source).unwrap();
        clock.sample(Duration::from_millis(150)).unwrap();
        assert!(matches!(
            clock.sample(Duration::from_millis(100)),
            Err(AnimationError::TimeWentBackwards { .. })
        ));
        let next = clock.sample(Duration::from_millis(200)).unwrap();
        assert_eq!(next.events.len(), 1);
        assert_eq!(next.events[0].frame_index, 2);
        clock.sample(Duration::from_secs(1)).unwrap();
        assert!(matches!(
            clock.sample(Duration::from_millis(900)),
            Err(AnimationError::TimeWentBackwards { .. })
        ));
    }

    #[test]
    fn skips_unknown_events_all_zero_and_invalid_fps_are_rejected() {
        for event in [
            FrameEvent::Skip,
            FrameEvent::UninterpretedNumeric {
                marker: b'^',
                value: 47,
            },
        ] {
            let mut source = animation(24, &[1, 1]);
            source.frames[1].events.push(event);
            assert!(matches!(
                AnimationClock::new(&source),
                Err(AnimationError::UnsupportedEvent {
                    frame_index: 1,
                    event_index: 1
                })
            ));
        }
        for fps in [0, 1001, u32::MAX] {
            assert!(matches!(
                AnimationClock::new(&animation(fps, &[1])),
                Err(AnimationError::InvalidFrameRate(_))
            ));
        }
        assert!(matches!(
            AnimationClock::new(&animation(24, &[0, 0])),
            Err(AnimationError::ZeroDuration)
        ));
        assert!(matches!(
            AnimationClock::new(&animation(24, &[])),
            Err(AnimationError::EmptyAnimation)
        ));
    }

    #[test]
    fn frame_and_event_budgets_are_validated_before_sampling() {
        let source = animation(24, &[1, 1, 1]);
        assert!(matches!(
            AnimationClock::with_limits(
                &source,
                PlaybackLimits {
                    max_frames: 2,
                    max_events: 10
                }
            ),
            Err(AnimationError::LimitExceeded("frames"))
        ));
        assert!(matches!(
            AnimationClock::with_limits(
                &source,
                PlaybackLimits {
                    max_frames: 3,
                    max_events: 2
                }
            ),
            Err(AnimationError::LimitExceeded("events"))
        ));
        assert!(matches!(
            AnimationClock::with_limits(
                &source,
                PlaybackLimits {
                    max_frames: usize::MAX,
                    max_events: usize::MAX
                }
            ),
            Err(AnimationError::InvalidLimits)
        ));
        let mut silent = animation(24, &[1]);
        silent.frames[0].events.clear();
        assert!(AnimationClock::with_limits(
            &silent,
            PlaybackLimits {
                max_frames: 1,
                max_events: 0
            }
        )
        .is_ok());
    }

    #[test]
    fn long_source_ticks_do_not_wrap_and_no_event_is_repeated_at_finish() {
        let source = animation(1, &[u32::MAX, u32::MAX]);
        let mut clock = AnimationClock::new(&source).unwrap();
        assert_eq!(clock.total_ticks(), u64::from(u32::MAX) * 2);
        let first = clock.sample(Duration::ZERO).unwrap();
        assert_eq!(
            first.next_deadline,
            Some(Duration::from_secs(u64::from(u32::MAX)))
        );
        let end = clock.sample(clock.duration().unwrap()).unwrap();
        assert!(end.finished);
        assert_eq!(end.events.len(), 1);
        assert_eq!(end.events[0].frame_index, 1);
        assert!(clock.sample(Duration::MAX).unwrap().events.is_empty());
    }

    #[test]
    fn clock_owns_only_a_validated_snapshot_and_does_not_borrow_the_catalog() {
        let mut source = animation(24, &[1, 1]);
        let mut clock = AnimationClock::new(&source).unwrap();
        source.frames[0].events.clear();
        source.frames[0].subframe = 999;
        drop(source);
        assert_eq!(clock.animation().frames_per_second, 24);
        let sample = clock.sample(Duration::ZERO).unwrap();
        assert_eq!(sample.subframe, 10);
        assert_eq!(sample.events[0].event, &FrameEvent::Sound(0));
        let mut oversized_label = animation(24, &[1]);
        oversized_label.label_bytes.resize(256 * 1024 + 1, b'x');
        assert!(matches!(
            AnimationClock::new(&oversized_label),
            Err(AnimationError::LimitExceeded("label bytes"))
        ));
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; verifies a real shared attack timeline"]
    fn real_shared_attack_uses_source_fps_ticks_and_once_only_presentation_events() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        use crate::modern_lito::sprite_actions::{ActionKey, ActionLimits, SpriteActionCatalog};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let bytes = SharedArchive::open(&root, SharedArchiveKind::Text)
            .unwrap()
            .read_entry("list.spr")
            .unwrap();
        let catalog =
            SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
        let attack = catalog.resolve_action(0, ActionKey::new(1)).unwrap();
        assert_eq!(attack.animation.frames_per_second, 24);
        assert_eq!(
            attack
                .animation
                .frames
                .iter()
                .map(|f| f.duration_ticks)
                .collect::<Vec<_>>(),
            vec![0, 3, 3, 3, 5, 2, 4]
        );
        let mut clock = AnimationClock::new(attack.animation).unwrap();
        drop(catalog);
        drop(bytes);
        let setup = clock.sample(Duration::ZERO).unwrap();
        assert_eq!(
            (setup.frame_index, setup.image_id, setup.subframe),
            (1, 16, 0)
        );
        assert!(setup.events.is_empty());
        assert_eq!(setup.next_deadline, Some(Duration::from_millis(125)));
        let sound = clock.sample(Duration::from_millis(375)).unwrap();
        assert_eq!((sound.frame_index, sound.subframe), (4, 3));
        assert_eq!(
            sound
                .events
                .iter()
                .map(|e| (e.tick, e.event))
                .collect::<Vec<_>>(),
            vec![(9, &FrameEvent::Sound(1539))]
        );
        let impact = clock.sample(Duration::from_nanos(583_333_334)).unwrap();
        assert_eq!((impact.frame_index, impact.subframe), (5, 4));
        assert_eq!(
            impact.events.iter().map(|e| e.event).collect::<Vec<_>>(),
            vec![&FrameEvent::Hit]
        );
        assert_eq!(
            impact.next_deadline,
            Some(Duration::from_nanos(666_666_667))
        );
        assert!(
            !clock
                .sample(Duration::from_nanos(833_333_333))
                .unwrap()
                .finished
        );
        let end = clock.sample(Duration::from_nanos(833_333_334)).unwrap();
        assert!(end.finished);
        assert!(end.events.is_empty());
        assert_eq!(end.frame_index, 6);
    }
}
