//! One bounded attachment sequence for independent canonical presentation banks.
//! Each producer retains its own monotonic cursor; no renumbering on repaint.
use super::world_audio::{AudioFrame, EventLog};
use anyhow::{ensure, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};

pub(crate) const SOURCES: usize = 8;

#[derive(Clone, Default)]
pub(crate) struct Mixer {
    cursors: [u64; SOURCES],
    log: EventLog,
    origins: BTreeMap<u64, (usize, u64)>,
    pub(crate) skipped: bool,
}
impl Mixer {
    #[cfg(test)]
    pub(crate) fn is_body_sequence(&self, sequence: u64) -> bool {
        self.origins
            .get(&sequence)
            .is_some_and(|(source, _)| *source == 6)
    }

    pub(crate) fn merge<const N: usize>(
        &mut self,
        frames: [Option<AudioFrame>; N],
        now: Instant,
    ) -> Result<Option<AudioFrame>> {
        ensure!(N <= SOURCES, "canonical audio source limit");
        let mut candidate = self.clone();
        let mut pending = Vec::new();
        let mut default_bank = None;
        let mut retained = BTreeSet::new();
        for (index, frame) in frames.into_iter().enumerate() {
            let Some(frame) = frame else {
                continue;
            };
            default_bank = Some(frame.bank.clone());
            let mut previous = 0;
            for event in frame.events.iter() {
                ensure!(
                    event.sequence > previous && event.created <= now,
                    "invalid canonical audio order/time"
                );
                previous = event.sequence;
                retained.insert((index, event.sequence));
                if event.sequence <= self.cursors[index] {
                    continue;
                }
                candidate.cursors[index] = event.sequence;
                if now.duration_since(event.created) <= Duration::from_millis(500) {
                    pending.push((
                        index,
                        event.clone(),
                        event.bank.clone().unwrap_or_else(|| frame.bank.clone()),
                    ));
                }
            }
        }
        pending.sort_by_key(|(_, event, _)| event.created);
        candidate.log.retain_sequences(|id| {
            candidate
                .origins
                .get(&id)
                .is_some_and(|origin| retained.contains(origin))
        });
        candidate.skipped |=
            candidate.log.snapshot(now).len() + pending.len() > super::world_audio::HISTORY_LIMIT;
        for (index, event, bank) in pending {
            candidate
                .log
                .push_with_bank(event.group, event.created, Some(bank))?;
            candidate
                .origins
                .insert(candidate.log.last_sequence(), (index, event.sequence));
        }
        // A producer may withdraw a queued cue (pet MOVE/finish/DNF). Keep
        // already assigned global IDs, but never re-expose a withdrawn cue.
        candidate.log.retain_sequences(|id| {
            candidate
                .origins
                .get(&id)
                .is_some_and(|origin| retained.contains(origin))
        });
        let events = candidate.log.snapshot(now);
        candidate
            .origins
            .retain(|id, _| events.iter().any(|event| event.sequence == *id));
        let result = default_bank.map(|bank| AudioFrame { bank, events });
        *self = candidate;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        combat_animation::classify_native_cues, combat_audio::SoundBank,
        sprite_actions::FrameEvent, world_audio::SoundEvent,
    };
    use std::{collections::BTreeMap, sync::Arc};
    fn frame(sequence: u64, created: Instant, id: u32) -> AudioFrame {
        let bank = Arc::new(SoundBank::playback_subset(BTreeMap::new()).unwrap());
        AudioFrame {
            bank,
            events: vec![SoundEvent {
                sequence,
                created,
                group: classify_native_cues(&[FrameEvent::Sound(id)])
                    .unwrap()
                    .audio
                    .unwrap(),
                bank: None,
            }]
            .into(),
        }
    }
    #[test]
    fn independent_sequence_one_cues_keep_banks_and_deduplicate_across_handoff() {
        let now = Instant::now();
        let first = frame(1, now, 97);
        let second = frame(1, now, 142);
        let third = frame(1, now, 109);
        let mut mixer = Mixer::default();
        let output = mixer
            .merge(
                [
                    Some(first.clone()),
                    Some(second.clone()),
                    Some(third.clone()),
                ],
                now,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            output.events.iter().map(|e| e.sequence).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert!(Arc::ptr_eq(
            output.events[0].bank.as_ref().unwrap(),
            &first.bank
        ));
        assert!(Arc::ptr_eq(
            output.events[1].bank.as_ref().unwrap(),
            &second.bank
        ));
        assert!(Arc::ptr_eq(
            output.events[2].bank.as_ref().unwrap(),
            &third.bank
        ));
        let mut replacement = mixer.clone();
        let same = replacement
            .merge([Some(first), Some(second), Some(third)], now)
            .unwrap()
            .unwrap();
        assert_eq!(same.events.len(), 3);
        assert_eq!(replacement.cursors, [1, 1, 1, 0, 0, 0, 0, 0]);
    }
    #[test]
    fn stale_future_or_unsorted_sources_cannot_poison_cursor() {
        let now = Instant::now();
        let mut mixer = Mixer::default();
        assert!(mixer
            .merge(
                [
                    Some(frame(7, now + Duration::from_nanos(1), 97)),
                    None,
                    None
                ],
                now
            )
            .is_err());
        assert_eq!(mixer.cursors, [0; SOURCES]);
        let mut bad = frame(2, now, 97);
        let mut events = bad.events.to_vec();
        events.push(events[0].clone());
        bad.events = events.into();
        assert!(mixer.merge([Some(bad), None, None], now).is_err());
        assert_eq!(mixer.cursors, [0; SOURCES]);
        let output = mixer
            .merge(
                [
                    Some(frame(7, now - Duration::from_millis(501), 97)),
                    None,
                    None,
                ],
                now,
            )
            .unwrap()
            .unwrap();
        assert!(output.events.is_empty());
        assert_eq!(mixer.cursors, [7, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn body_and_eighth_armor_source_can_share_a_sound_id_without_collision_or_resurrection() {
        let now = Instant::now();
        let frames: [Option<AudioFrame>; SOURCES] =
            std::array::from_fn(|_| Some(frame(1, now, 97)));
        let mut mixer = Mixer::default();
        let output = mixer.merge(frames.clone(), now).unwrap().unwrap();
        assert_eq!(output.events.len(), SOURCES);
        for (i, event) in output.events.iter().enumerate() {
            assert_eq!(event.sequence, i as u64 + 1);
            assert!(Arc::ptr_eq(
                event.bank.as_ref().unwrap(),
                &frames[i].as_ref().unwrap().bank
            ));
            assert_eq!(mixer.is_body_sequence(event.sequence), i == 6);
        }
        let mut withdrawn = frames.clone();
        withdrawn[6] = None;
        let without = mixer.merge(withdrawn, now).unwrap().unwrap();
        assert_eq!(without.events.len(), SOURCES - 1);
        let replay = mixer.merge(frames, now).unwrap().unwrap();
        assert_eq!(replay.events.len(), SOURCES - 1);
        assert_eq!(mixer.cursors, [1; SOURCES]);
    }

    #[test]
    fn late_healing_cue_does_not_outlive_its_age_behind_a_newer_spell() {
        let now = Instant::now();
        let spell = frame(1, now, 97);
        let healing = frame(1, now - Duration::from_millis(400), 109);
        let mut mixer = Mixer::default();
        mixer.merge([Some(spell.clone()), None, None], now).unwrap();
        let both = mixer
            .merge([Some(spell.clone()), None, Some(healing.clone())], now)
            .unwrap()
            .unwrap();
        assert_eq!(both.events.len(), 2);
        let later = now + Duration::from_millis(101);
        let recent = mixer
            .merge([Some(spell), None, Some(healing)], later)
            .unwrap()
            .unwrap();
        assert_eq!(recent.events.len(), 1);
        assert_eq!(recent.events[0].sequence, 1);
        assert_eq!(recent.events[0].group.candidates(), [97]);
        assert_eq!(mixer.cursors, [1, 0, 1, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn all_six_sources_keep_distinct_banks_and_withdraw_without_replay() {
        let now = Instant::now();
        let frames = [97, 121, 109, 119, 360, 86].map(|id| Some(frame(1, now, id)));
        let mut mixer = Mixer::default();
        let first = mixer.merge(frames.clone(), now).unwrap().unwrap();
        assert_eq!(first.events.len(), 6);
        assert_eq!(mixer.origins.len(), 6);
        for (event, source) in first.events.iter().zip(&frames) {
            assert!(Arc::ptr_eq(
                event.bank.as_ref().unwrap(),
                &source.as_ref().unwrap().bank
            ));
        }
        let mut withdrawn = frames.clone();
        withdrawn[5].as_mut().unwrap().events = Arc::from([]);
        let next = mixer.merge(withdrawn, now).unwrap().unwrap();
        assert_eq!(
            next.events.iter().map(|e| e.sequence).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(first.events.len(), 6, "published batch remains immutable");
        assert_eq!(
            mixer.merge(frames, now).unwrap().unwrap().events.len(),
            5,
            "withdrawn local sequence cannot return"
        );
        let silent: [Option<AudioFrame>; SOURCES] = Default::default();
        assert!(mixer.merge(silent, now).unwrap().is_none());
        assert!(mixer.origins.is_empty());
        assert_eq!(mixer.cursors, [1, 1, 1, 1, 1, 1, 0, 0]);
    }

    #[test]
    fn origin_tracking_is_bounded_and_invalid_last_source_is_atomic() {
        let now = Instant::now();
        let mut mixer = Mixer::default();
        let mut source = frame(1, now, 119);
        source.events = (1..=300)
            .map(|seq| frame(seq, now, 119).events[0].clone())
            .collect::<Vec<_>>()
            .into();
        let output = mixer.merge([Some(source.clone())], now).unwrap().unwrap();
        assert_eq!(output.events.len(), 256);
        assert!(
            mixer.skipped,
            "a truncated audio batch is not a complete presentation"
        );
        assert_eq!(mixer.origins.len(), 256);
        let saved = mixer.cursors;
        assert!(mixer
            .merge(
                [
                    Some(frame(301, now, 119)),
                    None,
                    None,
                    None,
                    None,
                    Some(frame(1, now + Duration::from_nanos(1), 360))
                ],
                now
            )
            .is_err());
        assert_eq!(mixer.cursors, saved);
        let same = mixer.merge([Some(source)], now).unwrap().unwrap();
        assert_eq!(same.events[0].sequence, 45);
        assert_eq!(same.events.last().unwrap().sequence, 300);
    }
}
