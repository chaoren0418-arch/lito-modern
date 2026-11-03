//! Confirmed canonical teleport presentation. No input lock, local movement,
//! packet conversion, generated revision, or LOAD/command authority.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    compositor::Canvas,
    paced_animation::PacedClock,
    teleport_effect::TeleportAssets,
    world_audio::{AudioFrame, EventLog},
    world_map::{WorldCell, WorldMap},
    world_scene::{ActorOverride, BaseAppearance, SceneSnapshot},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone)]
struct Event {
    revision: i64,
    life: i64,
    equipment: i64,
    sequence: i64,
    origin: WorldCell,
    destination: WorldCell,
    started: Instant,
    body: PacedClock,
    audio_emitted: bool,
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    observed: Instant,
    events: BTreeMap<i64, Event>,
    audio: EventLog,
}
fn cell(c: &d::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
fn recent(receipt: i64, observed: i64) -> bool {
    observed
        .checked_sub(receipt)
        .is_some_and(|age| (0..=500_000_000).contains(&age))
}
#[cfg(test)]
type TimingSample = (&'static str, i64, u128, u128);
#[cfg(test)]
fn timing_samples() -> &'static std::sync::Mutex<Vec<TimingSample>> {
    static SAMPLES: std::sync::OnceLock<std::sync::Mutex<Vec<TimingSample>>> = std::sync::OnceLock::new();
    SAMPLES.get_or_init(|| std::sync::Mutex::new(Vec::with_capacity(32)))
}
#[cfg(test)]
type FrameSample = (&'static str, i64, Vec<(u64, Vec<u32>, u128)>);
#[cfg(test)]
fn frame_samples() -> &'static std::sync::Mutex<Vec<FrameSample>> {
    static SAMPLES: std::sync::OnceLock<std::sync::Mutex<Vec<FrameSample>>> = std::sync::OnceLock::new();
    SAMPLES.get_or_init(|| std::sync::Mutex::new(Vec::with_capacity(24)))
}
#[cfg(test)]
pub(crate) fn frame_diagnostic(stage: &'static str, observation: &Observation, now: Instant, frame: Option<&AudioFrame>) {
    if std::env::var_os("LITO_MODERN_TEST_TELEPORT_DIAGNOSTICS").is_none()
        || !observation.data().actors.0.iter().any(|a| a.spells.histories.0.iter()
            .any(|s| s.lane == d::Lane::Teleport && s.receipt.is_some())) { return; }
    let mut samples = frame_samples().lock().unwrap();
    if samples.len() >= 24 { return; }
    let cues = frame.into_iter().flat_map(|f| f.events.iter())
        .map(|e| (e.sequence, e.group.candidates().to_vec(), now.saturating_duration_since(e.created).as_millis())).collect();
    samples.push((stage, observation.data().world.revision, cues));
}
#[cfg(test)]
fn timing_diagnostic(stage: &'static str, revision: i64, now: Instant, started: Instant) {
    if std::env::var_os("LITO_MODERN_TEST_TELEPORT_DIAGNOSTICS").is_none() { return; }
    let sample = (stage, revision, now.saturating_duration_since(started).as_millis(),
        Instant::now().saturating_duration_since(now).as_millis());
    let mut samples = timing_samples().lock().unwrap();
    if samples.len() < 32 { samples.push(sample); }
}
#[cfg(test)]
pub(crate) fn dump_timing_diagnostics() {
    let samples = std::mem::take(&mut *timing_samples().lock().unwrap());
    for (stage, revision, age, execution) in samples {
        eprintln!("TELEPORT_TIMELINE_TIMING stage={} revision={} event_age_ms={} clock_to_execution_ms={}", stage, revision, age, execution);
    }
    let frames = std::mem::take(&mut *frame_samples().lock().unwrap());
    for (stage, revision, cues) in frames {
        eprintln!("TELEPORT_AUDIO_FRAME stage={} revision={} sequence_candidates_age_ms={:?}", stage, revision, cues);
    }
}
#[cfg(test)]
pub(crate) fn scene_timing_diagnostic(stage: &'static str, observation: &Observation, started: Instant) {
    if observation.data().actors.0.iter().any(|a| a.spells.histories.0.iter()
        .any(|s| s.lane == d::Lane::Teleport && s.receipt.is_some())) {
        timing_diagnostic(stage, observation.data().world.revision, Instant::now(), started);
    }
}
pub(crate) fn enabled(observation: &Observation) -> bool {
    observation.data().actors.0.iter().any(|a| {
        a.spells
            .histories
            .0
            .iter()
            .any(|s| s.lane == d::Lane::Teleport)
    })
}
pub(crate) fn check_map(observation: &Observation, map: &WorldMap) -> Result<()> {
    for actor in &observation.data().actors.0 {
        for s in actor
            .spells
            .histories
            .0
            .iter()
            .filter(|s| s.lane == d::Lane::Teleport)
        {
            if let Some(r) = &s.receipt {
                let d::Detail::Teleport { destination } = &r.detail else {
                    anyhow::bail!("teleport receipt kind");
                };
                ensure!(
                    map.standing(cell(&r.impact.usage.before.cell))
                        && map.standing(cell(destination)),
                    "canonical teleport outside qualified map"
                );
            }
        }
    }
    Ok(())
}
impl Timeline {
    pub(crate) fn new(observation: Observation, now: Instant) -> Self {
        Self {
            observation,
            observed: now,
            events: BTreeMap::new(),
            audio: EventLog::default(),
        }
    }
    pub(crate) fn advance(
        &self,
        next: Observation,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        ensure!(
            !cancel.load(Ordering::Acquire) && now >= self.observed,
            "teleport cancelled/clock reversed"
        );
        self.observation.check_successor(&next)?;
        ensure!(
            next.data().world.phase != d::Phase::Aborted,
            "aborted teleport must retire"
        );
        let mut candidate = self.clone();
        let data = next.data();
        candidate.events.retain(|id, e| {
            now.saturating_duration_since(e.started) < Duration::from_secs(10)
                && data
                    .time_attack
                    .as_ref()
                    .is_none_or(|t| !t.departed.0.contains(id))
        });
        for a in &data.actors.0 {
            ensure!(!cancel.load(Ordering::Acquire), "teleport cancelled");
            if data
                .time_attack
                .as_ref()
                .is_some_and(|t| t.departed.0.contains(&a.entity))
            {
                continue;
            }
            if let Some(r) = a
                .spells
                .histories
                .0
                .iter()
                .find(|s| s.lane == d::Lane::Teleport)
                .and_then(|s| s.receipt.as_ref())
            {
                let usage = &r.impact.usage;
                // A fresh attachment never replays history. A skipped poll may
                // show only the latest, still-recent confirmed cast, not a burst.
                if usage.revision > self.observation.data().world.revision
                    && recent(r.loading_elapsed_nanos, data.world.loading_elapsed_nanos)
                {
                    let d::Detail::Teleport { destination } = &r.detail else {
                        anyhow::bail!("teleport receipt kind");
                    };
                    candidate.events.insert(
                        a.entity,
                        Event {
                            revision: usage.revision,
                            life: usage.before.life,
                            equipment: usage.before.equipment_revision,
                            sequence: a.last_move_sequence,
                            origin: cell(&usage.before.cell),
                            destination: cell(destination),
                            started: now
                                .checked_sub(Duration::from_nanos(u64::try_from(
                                    data.world.loading_elapsed_nanos - r.loading_elapsed_nanos,
                                )?))
                                .context("teleport timestamp underflow")?,
                            body: PacedClock::new(now),
                            audio_emitted: false,
                        },
                    );
                    #[cfg(test)]
                    timing_diagnostic("accepted", usage.revision, now, candidate.events[&a.entity].started);
                }
            }
            if let Some(event) = candidate.events.get_mut(&a.entity) {
                #[cfg(test)]
                timing_diagnostic("retime-before", event.revision, Instant::now(), now);
                let base = u64::try_from(a.cast_interval_nanos)?;
                ensure!(base > 0, "teleport cast interval absent");
                event
                    .body
                    .retime(now, base, super::battle_motion::paced_nanos(a, base, false))?;
                #[cfg(test)]
                timing_diagnostic("retime-after", event.revision, Instant::now(), now);
            }
        }
        candidate.observation = next;
        candidate.observed = now;
        #[cfg(test)]
        scene_timing_diagnostic("teleport-advance-return", &candidate.observation, now);
        Ok(candidate)
    }
    pub(crate) fn overrides<'a>(
        &self,
        scene: &SceneSnapshot,
        bank: &'a TeleportAssets,
        now: Instant,
    ) -> Result<Vec<ActorOverride<'a>>> {
        ensure!(now >= self.observed, "teleport sample clock reversed");
        let data = self.observation.data();
        let mut result = Vec::new();
        for a in &data.actors.0 {
            let Some(e) = self.events.get(&a.entity) else {
                continue;
            };
            let member = data.scene_member(a)?;
            let Some(resources) = &a.resources else {
                continue;
            };
            if scene.hidden(a.entity as u64)
                || a.vital.as_ref().is_none_or(|v| v.values.hp == 0)
                || super::battle_motion::life(a) != Some(e.life)
                || resources.equipment_revision != e.equipment
                || a.last_move_sequence != e.sequence
                || cell(&a.position) != e.destination
                || super::battle_motion::latest_body_event(data, a, member) > e.revision
            {
                continue;
            }
            let appearance = BaseAppearance {
                class: member.class.try_into()?,
                sex: member.sex.try_into()?,
            };
            let elapsed = e.body.peek(now)?;
            if elapsed >= bank.cast_duration(appearance)? {
                continue;
            }
            let sample = bank.caster_frame(appearance, a.heading.try_into()?, elapsed)?;
            if !sample.finished {
                result.push(ActorOverride {
                    entity_id: a.entity.try_into()?,
                    frame: sample.frame,
                    graphics_id: sample.graphics_id,
                    source_frame: sample.source_frame,
                    sprite_name: sample.sprite_name,
                    subframe: sample.subframe,
                });
            }
        }
        Ok(result)
    }
    pub(crate) fn draw(
        &self,
        canvas: &mut Canvas,
        projection: super::world_map::ViewProjection,
        bank: &TeleportAssets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        ensure!(now >= self.observed, "teleport sample clock reversed");
        for e in self.events.values() {
            ensure!(!cancel.load(Ordering::Acquire), "teleport cancelled");
            let elapsed = now.duration_since(e.started);
            if elapsed < bank.effect_duration() {
                let anchor = projection
                    .cell_to_pixel(e.origin)
                    .context("teleport projection overflow")?;
                bank.draw_effect(canvas, anchor, elapsed)?;
            }
        }
        Ok(())
    }
    pub(crate) fn audio(&mut self, bank: &TeleportAssets, now: Instant) -> Result<AudioFrame> {
        ensure!(now >= self.observed, "teleport audio clock reversed");
        let mut candidate = self.clone();
        for event in candidate.events.values_mut() {
            if !event.audio_emitted {
                #[cfg(test)]
                timing_diagnostic("first-audio-sample", event.revision, now, event.started);
                if now.duration_since(event.started) <= Duration::from_millis(500) {
                    candidate.audio.push_with_bank(
                        bank.sound_group().clone(),
                        event.started,
                        None,
                    )?;
                }
                event.audio_emitted = true;
            }
        }
        let output = AudioFrame {
            bank: bank.sound_bank(),
            events: candidate.audio.snapshot(now),
        };
        #[cfg(test)]
        frame_diagnostic("timeline-output", &candidate.observation, Instant::now(), Some(&output));
        *self = candidate;
        Ok(output)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    fn presentation_age_is_bounded_and_never_accepts_future_or_overflowed_receipts() {
        assert!(recent(1_000, 1_000));
        assert!(recent(1_000, 500_001_000));
        assert!(!recent(1_000, 500_001_001));
        assert!(!recent(1_000, 999));
        assert!(!recent(i64::MIN, i64::MAX));
    }

    pub(crate) fn verify_audio(base: Observation, used: Observation, bank: &TeleportAssets) {
        let at = Instant::now() + Duration::from_secs(30);
        let cancel = AtomicBool::new(false);
        let first = Timeline::new(base, at);
        assert!(first
            .advance(used.clone(), at, &AtomicBool::new(true))
            .is_err());
        assert!(first.events.is_empty());
        let mut accepted = first.advance(used.clone(), at, &cancel).unwrap();
        assert!(accepted.audio(bank, at - Duration::from_nanos(1)).is_err());
        assert!(accepted.events.values().all(|e| !e.audio_emitted));
        let cue = accepted.audio(bank, at).unwrap();
        assert_eq!(cue.events.len(), 1);
        assert_eq!(cue.events[0].group.candidates(), [119]);
        assert!(cue.bank.get(119).is_some());
        assert_eq!(cue.events[0].created, at);
        assert_eq!(accepted.audio(bank, at).unwrap().events[0].sequence, 1);
        assert!(Timeline::new(used.clone(), at)
            .audio(bank, at)
            .unwrap()
            .events
            .is_empty());
        let mut late = first.advance(used.clone(), at, &cancel).unwrap();
        assert!(late
            .audio(bank, at + Duration::from_millis(501))
            .unwrap()
            .events
            .is_empty());
        assert!(late
            .audio(bank, at + Duration::from_millis(502))
            .unwrap()
            .events
            .is_empty());
        for (delay, expected) in [(200_000_000, 1), (500_000_000, 1), (500_000_001, 0)] {
            let mut data = used.data().clone();
            data.world.loading_elapsed_nanos += delay;
            data.world.active_elapsed_nanos += delay;
            let next = super::super::battle_scene::tests::observation(data);
            let mut timeline = first.advance(next, at, &cancel).unwrap();
            let output = timeline.audio(bank, at).unwrap();
            assert_eq!(output.events.len(), expected);
            if expected == 1 {
                assert_eq!(
                    output.events[0].created,
                    at - Duration::from_nanos(delay as u64)
                );
                assert_eq!(timeline.events[&1].started, output.events[0].created);
            }
            if delay == 500_000_000 {
                assert!(timeline
                    .audio(bank, at + Duration::from_nanos(1))
                    .unwrap()
                    .events
                    .is_empty());
            }
        }
    }
}
