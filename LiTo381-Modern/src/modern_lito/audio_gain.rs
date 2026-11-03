//! Worker-only gain cache. Source WAVs stay immutable; the UI receives prepared
//! PCM and never copies/scales a buffer or changes the Windows device mixer.
use super::{
    combat_audio::{PreparedSound, SoundBank, MAX_BANK_BYTES, MAX_BANK_SOUNDS},
    world_audio::AudioFrame,
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Default)]
pub(crate) struct GainCache {
    gain: u8,
    sounds: BTreeMap<(u32, [u8; 32], Option<(u8, u8)>), PreparedSound>,
    bytes: usize,
}
impl GainCache {
    pub(crate) fn apply(
        &mut self,
        frame: AudioFrame,
        gain: u8,
        cancel: &AtomicBool,
    ) -> Result<AudioFrame> {
        ensure!(gain <= 100, "invalid frame gain");
        if self.gain != gain || gain == 0 || gain == 100 {
            self.sounds.clear();
            self.bytes = 0;
            self.gain = gain;
        }
        // Zero is a delivery gate, never a silent buffer queued to the driver.
        if gain == 0 || gain == 100 || frame.events.is_empty() {
            return Ok(frame);
        }
        let mut banks: Vec<(Arc<SoundBank>, BTreeSet<u32>)> =
            vec![(frame.bank.clone(), BTreeSet::new())];
        for event in frame.events.iter() {
            let bank = event.bank.as_ref().unwrap_or(&frame.bank);
            let index = match banks.iter().position(|(b, _)| Arc::ptr_eq(b, bank)) {
                Some(i) => i,
                None => {
                    banks.push((bank.clone(), BTreeSet::new()));
                    banks.len() - 1
                }
            };
            banks[index].1.extend(
                event
                    .group
                    .candidates()
                    .iter()
                    .copied()
                    .filter(|id| *id != 0),
            );
        }
        let mut unique = BTreeSet::new();
        let mut output_bytes = 0;
        let mut output = Vec::new();
        for (bank, ids) in &banks {
            let mut sounds = BTreeMap::new();
            for id in ids {
                ensure!(!cancel.load(Ordering::Acquire), "audio gain cancelled");
                let original = bank
                    .get(*id)
                    .context("gain bank misses a qualified candidate")?;
                let key = (*id, original.sha256(), original.spatial_gain());
                if unique.insert(key) {
                    output_bytes += original.pcm_bytes().len();
                    ensure!(
                        output_bytes <= MAX_BANK_BYTES && unique.len() <= MAX_BANK_SOUNDS,
                        "gain frame exceeds memory bound"
                    );
                }
                if !self.sounds.contains_key(&key) {
                    let bytes = original.pcm_bytes().len();
                    if self.bytes + bytes > MAX_BANK_BYTES || self.sounds.len() == MAX_BANK_SOUNDS {
                        self.sounds.clear();
                        self.bytes = 0;
                    }
                    let gained = original.with_gain(gain)?;
                    self.bytes += bytes;
                    self.sounds.insert(key, gained);
                }
                sounds.insert(*id, self.sounds[&key].clone());
            }
            output.push(Arc::new(SoundBank::playback_subset(sounds)?));
        }
        let events = frame
            .events
            .iter()
            .map(|event| {
                let mut event = event.clone();
                if let Some(bank) = &event.bank {
                    let index = banks
                        .iter()
                        .position(|(b, _)| Arc::ptr_eq(b, bank))
                        .unwrap();
                    event.bank = Some(output[index].clone());
                }
                event
            })
            .collect::<Vec<_>>()
            .into();
        Ok(AudioFrame {
            bank: output[0].clone(),
            events,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::modern_lito::{
        combat_animation::classify_native_cues, sprite_actions::FrameEvent, world_audio::SoundEvent,
    };
    pub(crate) fn wave(bits: u16, channels: u16, pcm: &[u8]) -> Vec<u8> {
        let mut b = b"RIFF".to_vec();
        b.extend((36 + pcm.len() as u32).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(channels.to_le_bytes());
        b.extend(22050u32.to_le_bytes());
        b.extend((22050u32 * u32::from(channels * bits / 8)).to_le_bytes());
        b.extend((channels * bits / 8).to_le_bytes());
        b.extend(bits.to_le_bytes());
        b.extend(b"data");
        b.extend((pcm.len() as u32).to_le_bytes());
        b.extend(pcm);
        b
    }
    #[test]
    fn pcm8_and_pcm16_mono_stereo_gain_is_exact_source_based_and_does_not_change_hash() {
        for channels in [1, 2] {
            let eight =
                PreparedSound::from_wav(97, wave(8, channels, &[0, 128, 255, 127])).unwrap();
            let sixteen = PreparedSound::from_wav(
                717,
                wave(16, channels, &[0, 128, 255, 127, 0, 0, 255, 255]),
            )
            .unwrap();
            for (source, expected) in [
                (&eight, vec![64, 128, 191, 128]),
                (&sixteen, vec![0, 192, 255, 63, 0, 0, 0, 0]),
            ] {
                let half = source.with_gain(50).unwrap();
                assert_eq!(half.pcm_bytes(), expected);
                assert_eq!(half.sha256(), source.sha256());
                assert_eq!(half.format(), source.format());
                assert_eq!(
                    half.with_gain(50).unwrap().pcm_bytes(),
                    expected,
                    "no compounded gain"
                );
                let restored = half.with_gain(100).unwrap();
                assert_eq!(restored.pcm_bytes(), source.pcm_bytes());
                assert_eq!(restored.pcm_bytes().as_ptr(), source.pcm_bytes().as_ptr());
                let muted = source.with_gain(0).unwrap();
                assert!(muted.pcm_bytes().iter().all(|v| *v
                    == if source.format().bits_per_sample == 8 {
                        128
                    } else {
                        0
                    }));
                assert!(source.with_gain(101).is_err());
            }
        }
    }
    fn bank(id: u32, pcm: &[u8]) -> Arc<SoundBank> {
        Arc::new(
            SoundBank::playback_subset(BTreeMap::from([(
                id,
                PreparedSound::from_wav(id, wave(8, 1, pcm)).unwrap(),
            )]))
            .unwrap(),
        )
    }
    #[test]
    fn gain_cache_preserves_event_scope_time_weighted_alternatives_and_distinct_same_id_resources()
    {
        let first = bank(97, &[0, 128, 255, 127]);
        let extra = bank(97, &[255, 128, 0, 129]);
        let now = std::time::Instant::now();
        let group = classify_native_cues(&[
            FrameEvent::Effect(97),
            FrameEvent::Effect(0),
            FrameEvent::Effect(97),
        ])
        .unwrap()
        .audio
        .unwrap();
        let frame = AudioFrame {
            bank: first.clone(),
            events: vec![
                SoundEvent {
                    sequence: 4,
                    created: now,
                    group: group.clone(),
                    bank: None,
                },
                SoundEvent {
                    sequence: 5,
                    created: now,
                    group,
                    bank: Some(extra),
                },
            ]
            .into(),
        };
        let mut cache = GainCache::default();
        let cancel = AtomicBool::new(false);
        let half = cache.apply(frame.clone(), 50, &cancel).unwrap();
        assert_eq!(half.bank.get(97).unwrap().pcm_bytes(), [64, 128, 191, 128]);
        assert_eq!(
            half.events[1]
                .bank
                .as_ref()
                .unwrap()
                .get(97)
                .unwrap()
                .pcm_bytes(),
            [191, 128, 64, 128]
        );
        for (a, b) in frame.events.iter().zip(half.events.iter()) {
            assert_eq!(a.sequence, b.sequence);
            assert_eq!(a.created, b.created);
            assert_eq!(a.group, b.group);
        }
        let repeated = cache.apply(frame.clone(), 50, &cancel).unwrap();
        assert_eq!(
            repeated.bank.get(97).unwrap().pcm_bytes().as_ptr(),
            half.bank.get(97).unwrap().pcm_bytes().as_ptr()
        );
        let full = cache.apply(frame.clone(), 100, &cancel).unwrap();
        assert!(Arc::ptr_eq(&full.bank, &first));
        assert_eq!(cache.bytes, 0);
        cancel.store(true, Ordering::Release);
        assert!(cache.apply(frame, 50, &cancel).is_err());
    }
    #[test]
    fn spatial_mono_stereo_gain_composes_from_source_without_format_or_cache_aliasing() {
        let source = PreparedSound::from_wav(478, wave(8, 1, &[0, 128, 255, 127])).unwrap();
        let left = source.with_spatial(127, 0).unwrap();
        assert_eq!(left.pcm_bytes(), [0, 128, 128, 128, 255, 128, 127, 128]);
        assert_eq!(left.format().channels, 2);
        assert_eq!(left.format().block_align, 2);
        assert_eq!(left.format().byte_rate, 44100);
        let near = source.with_spatial(116, 46).unwrap();
        assert_eq!(near.pcm_bytes(), [12, 44, 128, 128, 244, 211, 128, 128]);
        let half = near.with_gain(50).unwrap();
        assert_eq!(half.pcm_bytes(), [70, 86, 128, 128, 186, 169, 128, 128]);
        assert_eq!(half.with_gain(50).unwrap().pcm_bytes(), half.pcm_bytes());
        assert_eq!(half.with_gain(100).unwrap().pcm_bytes(), near.pcm_bytes());
        assert_eq!(half.sha256(), source.sha256());
        assert_eq!(half.file_bytes(), source.file_bytes());
        let sixteen = PreparedSound::from_wav(479, wave(16, 2, &[0, 128, 255, 127])).unwrap();
        assert_eq!(
            sixteen.with_spatial(127, 0).unwrap().pcm_bytes(),
            [0, 128, 0, 0]
        );
        assert_eq!(
            sixteen.with_spatial(127, 128).unwrap().pcm_bytes(),
            [0, 0, 255, 127]
        );
        assert_eq!(
            sixteen
                .with_spatial(127, 0)
                .unwrap()
                .with_gain(50)
                .unwrap()
                .pcm_bytes(),
            [0, 192, 0, 0]
        );
        assert!(source.with_spatial(128, 64).is_err());
        assert!(source.with_spatial(127, 129).is_err());
        let now = std::time::Instant::now();
        let group = classify_native_cues(&[FrameEvent::Effect(478)])
            .unwrap()
            .audio
            .unwrap();
        let first = Arc::new(SoundBank::playback_subset(BTreeMap::from([(478, left)])).unwrap());
        let second = Arc::new(SoundBank::playback_subset(BTreeMap::from([(478, near)])).unwrap());
        let frame = AudioFrame {
            bank: first.clone(),
            events: vec![
                SoundEvent {
                    sequence: 1,
                    created: now,
                    group: group.clone(),
                    bank: Some(first),
                },
                SoundEvent {
                    sequence: 2,
                    created: now,
                    group,
                    bank: Some(second),
                },
            ]
            .into(),
        };
        let mut cache = GainCache::default();
        let scaled = cache
            .apply(frame.clone(), 50, &AtomicBool::new(false))
            .unwrap();
        assert_eq!(
            cache.sounds.len(),
            2,
            "same WAV at different positions cannot alias"
        );
        assert_eq!(
            scaled.events[0]
                .bank
                .as_ref()
                .unwrap()
                .get(478)
                .unwrap()
                .pcm_bytes(),
            [64, 128, 128, 128, 191, 128, 128, 128]
        );
        assert_eq!(
            scaled.events[1]
                .bank
                .as_ref()
                .unwrap()
                .get(478)
                .unwrap()
                .pcm_bytes(),
            half.pcm_bytes()
        );
        let repeat = cache.apply(frame, 50, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            scaled.events[1]
                .bank
                .as_ref()
                .unwrap()
                .get(478)
                .unwrap()
                .pcm_bytes()
                .as_ptr(),
            repeat.events[1]
                .bank
                .as_ref()
                .unwrap()
                .get(478)
                .unwrap()
                .pcm_bytes()
                .as_ptr()
        );
    }
}
