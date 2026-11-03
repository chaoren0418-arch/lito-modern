//! Cues follow the exact visible objective clock, never HP heuristics or a
//! converted legacy frame. Historical initial states do not start playback.
use super::ObjectiveClock;
use crate::modern_lito::{
    tower_assets::{HonorPlayback, TowerAssets},
    world_audio::{crossed_audio_groups, AudioFrame, EventLog},
};
use anyhow::{ensure, Context, Result};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone)]
struct Active {
    clock: ObjectiveClock,
    playback: HonorPlayback,
}
impl Active {
    fn matches(&self, clock: &ObjectiveClock) -> bool {
        self.clock.target == clock.target
            && self.clock.graphic == clock.graphic
            && self.clock.cell == clock.cell
            && self.clock.action == clock.action
            && self.clock.started == clock.started
    }
}

#[derive(Clone, Default)]
pub(super) struct Timeline {
    active: Vec<Active>,
    log: EventLog,
    sampled: Option<Instant>,
}
impl Timeline {
    pub(super) fn sample(
        &mut self,
        clocks: &[ObjectiveClock],
        bank: &TowerAssets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<AudioFrame> {
        ensure!(
            !cancel.load(Ordering::Acquire) && self.sampled.is_none_or(|t| now >= t),
            "objective audio cancelled/clock reversed"
        );
        let mut candidate = self.clone();
        candidate
            .active
            .retain(|old| clocks.iter().any(|clock| old.matches(clock)));
        for clock in clocks {
            ensure!(!cancel.load(Ordering::Acquire), "objective audio cancelled");
            let Some(started) = clock.started else {
                continue;
            };
            let index = match candidate.active.iter().position(|old| old.matches(clock)) {
                Some(index) => index,
                None => {
                    candidate.active.push(Active {
                        clock: clock.clone(),
                        playback: bank.start_state(clock.graphic, clock.action)?,
                    });
                    candidate.active.len() - 1
                }
            };
            let sample = candidate.active[index]
                .playback
                .advance(clock.elapsed(now)?)?;
            for (tick, group) in crossed_audio_groups(&sample.crossed_cues)? {
                // Tower qualification pins 24 FPS; ceil exactly as OneShot.
                let nanos = (u128::from(tick) * 1_000_000_000).div_ceil(24);
                let created = started
                    .checked_add(Duration::from_nanos(u64::try_from(nanos)?))
                    .context("objective cue timestamp overflow")?;
                if now
                    .checked_duration_since(created)
                    .is_some_and(|age| age <= Duration::from_millis(500))
                {
                    ensure!(
                        group
                            .candidates()
                            .iter()
                            .all(|id| *id == 0 || bank.sound_bank().get(*id).is_some()),
                        "objective cue absent from qualified bank"
                    );
                    candidate.log.push_with_bank(group, created, None)?;
                }
            }
        }
        let output = AudioFrame {
            bank: bank.sound_bank(),
            events: candidate.log.snapshot(now),
        };
        candidate.sampled = Some(now);
        *self = candidate;
        Ok(output)
    }
}

#[cfg(test)]
pub(super) fn verify(clocks: &[ObjectiveClock], bank: &TowerAssets, now: Instant) {
    let clocks = clocks
        .iter()
        .filter(|c| c.action == 35 && c.started == Some(now))
        .cloned()
        .collect::<Vec<_>>();
    assert!(!clocks.is_empty());
    let cancel = AtomicBool::new(false);
    let mut timeline = Timeline::default();
    let mut bad = clocks.clone();
    let mut future = clocks[0].clone();
    future.started = Some(now + Duration::from_nanos(1));
    bad.push(future);
    assert!(timeline.sample(&bad, bank, now, &cancel).is_err());
    assert!(timeline.active.is_empty() && timeline.sampled.is_none());
    assert!(timeline
        .sample(&clocks, bank, now, &AtomicBool::new(true))
        .is_err());
    let first = timeline.sample(&clocks, bank, now, &cancel).unwrap();
    assert_eq!(first.events.len(), clocks.len());
    for event in first.events.iter() {
        assert_eq!(event.created, now);
        assert_eq!(event.group.candidates(), [360]);
        assert!(first.bank.get(360).is_some());
    }
    assert!(timeline
        .sample(&clocks, bank, now - Duration::from_nanos(1), &cancel)
        .is_err());
    let mut copy = timeline.clone();
    assert_eq!(
        copy.sample(&clocks, bank, now, &cancel)
            .unwrap()
            .events
            .len(),
        first.events.len()
    );
    let boundary = now + Duration::from_millis(500);
    assert_eq!(
        copy.sample(&clocks, bank, boundary, &cancel)
            .unwrap()
            .events
            .len(),
        first.events.len()
    );
    assert!(copy
        .sample(&clocks, bank, boundary + Duration::from_nanos(1), &cancel)
        .unwrap()
        .events
        .is_empty());
    let mut late = Timeline::default();
    assert!(late
        .sample(&clocks, bank, boundary + Duration::from_nanos(1), &cancel)
        .unwrap()
        .events
        .is_empty());
    let historical = clocks
        .iter()
        .cloned()
        .map(|mut c| {
            c.started = None;
            c
        })
        .collect::<Vec<_>>();
    assert!(Timeline::default()
        .sample(&historical, bank, now, &cancel)
        .unwrap()
        .events
        .is_empty());
    assert_eq!(
        first.events[0].sequence, 1,
        "failed candidate consumed nothing"
    );
}

#[cfg(test)]
#[test]
#[ignore = "requires real shared tower SPR/PCM; explicit local objective clocks, no Engine/TLS/GUI/device claim"]
fn actual_shared_objective_cues_are_atomic_finite_and_age_bounded() {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
    );
    let bank = TowerAssets::load(&root, &AtomicBool::new(false)).unwrap();
    let now = Instant::now();
    let clocks = crate::modern_lito::tower_assets::GRAPHICS
        .iter()
        .enumerate()
        .map(|(index, graphic)| ObjectiveClock {
            target: super::ObjectiveTarget::Honor {
                entity: 3 + index as i64,
                team: (index % 2) as i32,
            },
            graphic: *graphic,
            cell: crate::modern_lito::world_map::WorldCell {
                x: 100 + index as i32,
                y: 100,
            },
            action: 35,
            started: Some(now),
        })
        .collect::<Vec<_>>();
    verify(&clocks, &bank, now);
}
