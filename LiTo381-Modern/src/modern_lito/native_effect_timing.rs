//! Native finite-effect callbacks use integer milliseconds and reschedule from dispatch.
//! Explicit modern action clips retain their separate rational timeline.
use super::{combat_animation::CrossedCue, sprite_actions::Animation};
use anyhow::{ensure, Result};
use std::sync::Arc;

/// Stateless inspection of a rational 24Hz timeline, not native timer dispatch.
pub fn sample_ticks(ticks: &[u64], elapsed_ns: u64) -> (usize, bool) {
    let tick = u128::from(elapsed_ns) * 24 / 1_000_000_000;
    let mut end = 0u128;
    for source in (1..ticks.len()).chain((!ticks.is_empty()).then_some(0)) {
        end += u128::from(ticks[source]);
        if tick < end { return (source, false); }
    }
    (0, true)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Clock { NativeFinite, ExplicitAction }

#[derive(Clone)]
pub struct Clip {
    animation: Arc<Animation>,
    ticks: Vec<u64>,
    delays_ms: Vec<u64>,
    entry_ticks: Vec<u64>,
    clock: Clock,
}
impl Clip {
    pub fn new(animation: Arc<Animation>) -> Result<Self> {
        Self::with_clock(animation, Clock::NativeFinite)
    }
    /// The Life Stream action-3 presentation is explicitly modern, not class-0.
    pub fn new_explicit_action(animation: Arc<Animation>) -> Result<Self> {
        Self::with_clock(animation, Clock::ExplicitAction)
    }
    fn with_clock(animation: Arc<Animation>, clock: Clock) -> Result<Self> {
        ensure!((1..=256).contains(&animation.frames.len()), "location frame count");
        ensure!(animation.frames_per_second == 24, "location FPS");
        let ticks: Vec<_> = animation.frames.iter().map(|f| u64::from(f.duration_ticks)).collect();
        ensure!((1..=24 * 60).contains(&ticks.iter().sum::<u64>()), "location duration");
        ensure!(animation.frames.iter().map(|f| f.events.len()).sum::<usize>() <= 1024, "location cue budget");
        // PE4271D92F 43E888..43E907 stores each ticks*1000/FPS quotient,
        // separately, in the source frame. Do not round a cumulative tick sum.
        let delays_ms = ticks.iter().map(|t| t * 1000 / 24).collect();
        let mut entry_ticks = vec![0; ticks.len()];
        let mut at = 0;
        for source in (1..ticks.len()).chain(std::iter::once(0)) {
            entry_ticks[source] = at;
            at += ticks[source];
        }
        Ok(Self { animation, ticks, delays_ms, entry_ticks, clock })
    }
    pub fn start(&self) -> Playback {
        let frame = if self.ticks.len() == 1 { 0 } else { 1 };
        Playback { clip: self.clone(), last_ns: None, frame,
            next_ms: Some(self.delays_ms[frame]), published_initial: false,
            finished: false, callbacks: 1 }
    }
}
pub struct Sample {
    pub source_frame: usize,
    pub finished: bool,
    pub crossed_cues: Vec<CrossedCue>,
}
#[derive(Clone)]
pub struct Playback {
    clip: Clip,
    last_ns: Option<u64>,
    frame: usize,
    next_ms: Option<u64>,
    published_initial: bool,
    finished: bool,
    callbacks: u64,
}
impl Playback {
    pub fn next_callback_millis(&self) -> Option<u64> {
        (self.clip.clock == Clock::NativeFinite).then_some(self.next_ms).flatten()
    }
    /// Last dispatched clock sample, independent of a later diagnostic draw.
    pub fn sampled_native_millis(&self) -> Option<u64> {
        (self.clip.clock == Clock::NativeFinite).then_some(self.last_ns).flatten().map(|ns| ns / 1_000_000)
    }
    pub fn callback_count(&self) -> Option<u64> {
        (self.clip.clock == Clock::NativeFinite).then_some(self.callbacks)
    }
    fn enter(&self, source: usize, cues: &mut Vec<CrossedCue>) {
        for (event_index, event) in self.clip.animation.frames[source].events.iter().enumerate() {
            cues.push(CrossedCue { source_frame: source, event_index,
                tick: self.clip.entry_ticks[source], event: event.clone() });
        }
    }
    pub fn advance(&mut self, elapsed_ns: u64) -> Result<Sample> {
        ensure!(self.last_ns.is_none_or(|last| elapsed_ns >= last), "location clock regressed");
        if self.clip.clock == Clock::ExplicitAction {
            return self.advance_explicit(elapsed_ns);
        }
        let now_ms = elapsed_ns / 1_000_000;
        let mut crossed_cues = Vec::new();
        if !self.published_initial {
            self.enter(self.frame, &mut crossed_cues);
            self.published_initial = true;
        }
        let mut dispatched = 0;
        while self.next_ms.is_some_and(|due| now_ms >= due) {
            dispatched += 1;
            ensure!(dispatched <= self.clip.ticks.len(), "location zero-delay callback bound");
            self.callbacks += 1;
            if self.frame == 0 {
                self.finished = true;
                self.next_ms = None;
                break;
            }
            self.frame = if self.frame + 1 == self.clip.ticks.len() { 0 } else { self.frame + 1 };
            self.enter(self.frame, &mut crossed_cues);
            // 44F0DF/4D65FE: a late callback starts the next dwell at NOW.
            // Positive delays therefore cannot be caught up by this same poll.
            // Zero-delay frames remain bounded and preserve their cue order.
            self.next_ms = Some(now_ms + self.clip.delays_ms[self.frame]);
        }
        self.last_ns = Some(elapsed_ns);
        Ok(Sample { source_frame: self.frame, finished: self.finished, crossed_cues })
    }
    fn advance_explicit(&mut self, elapsed_ns: u64) -> Result<Sample> {
        let to = u128::from(elapsed_ns) * 24 / 1_000_000_000;
        let from = self.last_ns.map(|ns| u128::from(ns) * 24 / 1_000_000_000);
        let mut tick = 0u128;
        let mut crossed_cues = Vec::new();
        for source in (1..self.clip.ticks.len()).chain(std::iter::once(0)) {
            if tick <= to && from.is_none_or(|last| tick > last) { self.enter(source, &mut crossed_cues); }
            tick += u128::from(self.clip.ticks[source]);
        }
        let (source_frame, finished) = sample_ticks(&self.clip.ticks, elapsed_ns);
        self.last_ns = Some(elapsed_ns);
        Ok(Sample { source_frame, finished, crossed_cues })
    }
}
