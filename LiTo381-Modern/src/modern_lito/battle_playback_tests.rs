use super::*;
use crate::modern_lito::{
    audio_gain::tests::wave,
    combat_animation::classify_native_cues,
    combat_audio::PreparedSound,
    preferences::{Preferences, SharedPreferences},
    sprite_actions::FrameEvent,
    world_audio::SoundEvent,
};
use std::collections::BTreeMap;

fn attachment() -> Arc<Attachment> {
    Attachment::new(SharedPreferences::default().snapshot())
}

#[test]
fn first_tick_after_settings_changed_cannot_play_old_cues_reprepared_at_new_gain() {
    let owner = attachment();
    let at = Instant::now();
    let preferences = SharedPreferences::default();
    preferences
        .apply(Preferences {
            sound_enabled: false,
            ..Preferences::default()
        })
        .unwrap();
    preferences.apply(Preferences::default()).unwrap();
    let ready = prepared(frame(1, at, bank(&[0, 255])), &preferences);
    let mut gate = delivery();
    let now = at + Duration::from_millis(50);
    assert_eq!(
        gate.tick(view(&owner, Some(&ready), &preferences), true, now)
            .started,
        0
    );
    let fresh = prepared(frame(2, now, bank(&[0, 255])), &preferences);
    assert_eq!(
        gate.tick(view(&owner, Some(&fresh), &preferences), true, now)
            .started,
        1
    );
}

#[test]
fn reversed_tick_is_strict_and_does_not_rewind_or_poison_a_valid_next_batch() {
    let owner = attachment();
    let at = Instant::now();
    let preferences = SharedPreferences::default();
    let ready = prepared(frame(1, at, bank(&[0, 255])), &preferences);
    let mut gate = delivery();
    assert_eq!(
        gate.tick(view(&owner, Some(&ready), &preferences), true, at)
            .started,
        1
    );
    let next = prepared(frame(2, at, bank(&[0, 255])), &preferences);
    let refused = gate.tick(
        view(&owner, Some(&next), &preferences),
        true,
        at - Duration::from_nanos(1),
    );
    assert_eq!(refused.started, 0);
    assert!(refused.notice.is_some());
    assert_eq!(gate.sequence, 1);
    assert_eq!(
        gate.tick(view(&owner, Some(&next), &preferences), true, at)
            .started,
        1
    );
}

#[derive(Default)]
struct RecordingSink {
    polls: usize,
    stops: usize,
    played: Vec<Vec<u8>>,
    fail: bool,
}
impl Sink for RecordingSink {
    fn poll(&mut self) -> Result<()> {
        self.polls += 1;
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        self.stops += 1;
        Ok(())
    }
    fn play(&mut self, bank: &SoundBank, group: &NativeAudioGroup, random: u32) -> Result<bool> {
        anyhow::ensure!(!self.fail, "explicit synthetic driver failure");
        anyhow::ensure!(
            group
                .candidates()
                .iter()
                .all(|id| *id == 0 || bank.get(*id).is_some()),
            "missing test PCM"
        );
        let Some(id) = group.selected(random) else {
            return Ok(false);
        };
        self.played.push(bank.get(id).unwrap().pcm_bytes().to_vec());
        Ok(true)
    }
}
pub(crate) struct Recorder(Delivery<RecordingSink>);
impl Recorder {
    pub(crate) fn new() -> Self {
        Self(Delivery::new(RecordingSink::default(), 7))
    }
    pub(crate) fn tick(
        &mut self,
        runtime: &crate::modern_lito::battle_runtime::BattleRuntime,
        focused: bool,
        now: Instant,
    ) -> PlaybackStatus {
        self.0.tick(runtime.audio_view(), focused, now)
    }
    pub(crate) fn count(&self) -> usize {
        self.0.sink.played.len()
    }
}
pub(crate) fn bank(pcm: &[u8]) -> Arc<SoundBank> {
    Arc::new(
        SoundBank::playback_subset(BTreeMap::from([(
            97,
            PreparedSound::from_wav(97, wave(8, 1, pcm)).unwrap(),
        )]))
        .unwrap(),
    )
}
pub(crate) fn frame(sequence: u64, now: Instant, bank: Arc<SoundBank>) -> AudioFrame {
    AudioFrame {
        bank,
        events: vec![SoundEvent {
            sequence,
            created: now,
            group: classify_native_cues(&[FrameEvent::Sound(97)])
                .unwrap()
                .audio
                .unwrap(),
            bank: None,
        }]
        .into(),
    }
}
fn prepared(frame: AudioFrame, preferences: &SharedPreferences) -> PreparedAudio {
    PreparedAudio::prepare(
        Some(frame),
        preferences.snapshot(),
        &mut GainCache::default(),
        &AtomicBool::new(false),
    )
}
fn view<'a>(
    attachment: &'a Arc<Attachment>,
    prepared: Option<&'a PreparedAudio>,
    preferences: &SharedPreferences,
) -> View<'a> {
    View {
        attachment: Some(attachment),
        publication: prepared.map(|p| (attachment, p)),
        preferences: preferences.snapshot(),
        playable: true,
    }
}
fn delivery() -> Delivery<RecordingSink> {
    Delivery::new(RecordingSink::default(), 7)
}

#[test]
fn seven_same_id_banks_keep_pcm_gain_time_sequences_and_repaint_deduplication() {
    let attachment = attachment();
    let at = Instant::now();
    let preferences = SharedPreferences::default();
    preferences
        .apply(Preferences {
            volume_percent: 50,
            ..Preferences::default()
        })
        .unwrap();
    let banks = (0..7)
        .map(|i| bank(&[i * 2, 128, 254 - i * 2, 128]))
        .collect::<Vec<_>>();
    let source = AudioFrame {
        bank: banks[0].clone(),
        events: banks
            .iter()
            .enumerate()
            .map(|(i, bank)| {
                let mut e = frame(i as u64 + 1, at, bank.clone()).events[0].clone();
                e.bank = Some(bank.clone());
                e
            })
            .collect::<Vec<_>>()
            .into(),
    };
    let ready = prepared(source.clone(), &preferences);
    let mut gate = delivery();
    assert_eq!(
        gate.tick(view(&attachment, Some(&ready), &preferences), true, at)
            .started,
        7
    );
    for (i, pcm) in gate.sink.played.iter().enumerate() {
        assert_eq!(*pcm, vec![64 + i as u8, 128, 191 - i as u8, 128]);
        let actual = &ready.frame.as_ref().unwrap().events[i];
        assert_eq!(actual.sequence, source.events[i].sequence);
        assert_eq!(actual.created, source.events[i].created);
        assert_eq!(
            actual.bank.as_ref().unwrap().get(97).unwrap().sha256(),
            banks[i].get(97).unwrap().sha256()
        );
    }
    assert_eq!(
        gate.tick(view(&attachment, Some(&ready), &preferences), true, at)
            .started,
        0
    );
    assert_eq!(gate.sink.polls, 2);
}

#[test]
fn settings_aba_and_stale_preparation_consume_without_replaying_on_recovery() {
    let attachment = attachment();
    let at = Instant::now();
    let preferences = SharedPreferences::default();
    let old = prepared(frame(1, at, bank(&[0, 255])), &preferences);
    let mut gate = delivery();
    gate.tick(view(&attachment, None, &preferences), true, at);
    preferences
        .apply(Preferences {
            sound_enabled: false,
            ..Preferences::default()
        })
        .unwrap();
    preferences.apply(Preferences::default()).unwrap();
    let later = at + Duration::from_millis(10);
    assert_eq!(
        gate.tick(view(&attachment, Some(&old), &preferences), true, later)
            .started,
        0
    );
    assert_eq!(gate.sequence, 1);
    let reprepared = prepared(old.frame.unwrap(), &preferences);
    assert_eq!(
        gate.tick(
            view(&attachment, Some(&reprepared), &preferences),
            true,
            later
        )
        .started,
        0
    );
    let next = prepared(frame(2, later, bank(&[0, 255])), &preferences);
    assert_eq!(
        gate.tick(view(&attachment, Some(&next), &preferences), true, later)
            .started,
        1
    );
    assert_eq!(gate.sink.played.len(), 1);
}

#[test]
fn unfocused_or_muted_ticks_poll_and_consume_but_focus_return_drops_old_queued_cues() {
    let attachment = attachment();
    let at = Instant::now();
    let preferences = SharedPreferences::default();
    preferences
        .apply(Preferences {
            background_sound: false,
            ..Preferences::default()
        })
        .unwrap();
    let mut gate = delivery();
    let first = prepared(frame(1, at, bank(&[0, 255])), &preferences);
    assert_eq!(
        gate.tick(view(&attachment, Some(&first), &preferences), true, at)
            .started,
        1
    );
    let later = at + Duration::from_millis(20);
    let hidden = prepared(frame(2, later, bank(&[0, 255])), &preferences);
    assert_eq!(
        gate.tick(view(&attachment, Some(&hidden), &preferences), false, later)
            .started,
        0
    );
    assert_eq!(gate.sequence, 2);
    let return_at = later + Duration::from_millis(20);
    let late_hidden = prepared(frame(3, later, bank(&[0, 255])), &preferences);
    assert_eq!(
        gate.tick(
            view(&attachment, Some(&late_hidden), &preferences),
            true,
            return_at
        )
        .started,
        0
    );
    let visible = prepared(frame(4, return_at, bank(&[0, 255])), &preferences);
    assert_eq!(
        gate.tick(
            view(&attachment, Some(&visible), &preferences),
            true,
            return_at
        )
        .started,
        1
    );
    assert_eq!(gate.sink.polls, 4);
    assert_eq!(gate.sink.stops, 3);
}

#[test]
fn detached_and_wrong_attachment_batches_cannot_poison_new_sequence_one() {
    let old = attachment();
    let at = Instant::now();
    let preferences = SharedPreferences::default();
    let first = prepared(frame(20, at, bank(&[0, 255])), &preferences);
    let mut gate = delivery();
    assert_eq!(
        gate.tick(view(&old, Some(&first), &preferences), true, at)
            .started,
        1
    );
    gate.tick(
        View {
            attachment: None,
            publication: Some((&old, &first)),
            preferences: preferences.snapshot(),
            playable: true,
        },
        true,
        at,
    );
    let next = attachment();
    let now = Instant::now().max(at);
    assert_eq!(
        gate.tick(
            View {
                attachment: Some(&next),
                publication: Some((&old, &first)),
                preferences: preferences.snapshot(),
                playable: true
            },
            true,
            now
        )
        .started,
        0
    );
    assert_eq!(gate.sequence, 0);
    let fresh = prepared(frame(1, now, bank(&[0, 255])), &preferences);
    assert_eq!(
        gate.tick(view(&next, Some(&fresh), &preferences), true, now)
            .started,
        1
    );
}

#[test]
fn invalid_batch_is_atomic_future_one_ns_and_capacity_cannot_later_replay() {
    for kind in 0..3 {
        let owner = attachment();
        let at = Instant::now();
        let preferences = SharedPreferences::default();
        let mut input = frame(1, at, bank(&[0, 255]));
        let mut events = input.events.to_vec();
        let mut second = events[0].clone();
        second.sequence = 2;
        if kind == 0 {
            second.created = at + Duration::from_nanos(1);
        }
        if kind == 1 {
            second.sequence = 1;
        }
        events.push(second);
        if kind == 2 {
            events = (1..=257)
                .map(|sequence| SoundEvent {
                    sequence,
                    ..events[0].clone()
                })
                .collect();
        }
        input.events = events.into();
        let ready = prepared(input, &preferences);
        let mut gate = delivery();
        let result = gate.tick(view(&owner, Some(&ready), &preferences), true, at);
        assert_eq!(result.started, 0);
        assert!(result.notice.is_some());
        assert!(gate.sink.played.is_empty());
        let max = gate.sequence;
        let replay = prepared(frame(max, at, bank(&[0, 255])), &preferences);
        assert_eq!(
            gate.tick(view(&owner, Some(&replay), &preferences), true, at)
                .started,
            0
        );
    }
}

#[test]
fn exact_age_boundary_preparation_failure_and_loading_are_silent_consumers() {
    for age in [
        Duration::from_millis(500),
        Duration::from_millis(500) + Duration::from_nanos(1),
    ] {
        let owner = attachment();
        let at = Instant::now();
        let preferences = SharedPreferences::default();
        let ready = prepared(frame(1, at, bank(&[0, 255])), &preferences);
        let mut gate = delivery();
        assert_eq!(
            gate.tick(view(&owner, Some(&ready), &preferences), true, at + age)
                .started,
            usize::from(age == Duration::from_millis(500))
        );
    }
    let owner = attachment();
    let at = Instant::now();
    let preferences = SharedPreferences::default();
    preferences
        .apply(Preferences {
            volume_percent: 50,
            ..Preferences::default()
        })
        .unwrap();
    let empty = Arc::new(SoundBank::playback_subset(BTreeMap::new()).unwrap());
    let broken = prepared(frame(1, at, empty), &preferences);
    assert!(broken.gain.is_none() && broken.notice.is_some());
    let mut gate = delivery();
    assert_eq!(
        gate.tick(view(&owner, Some(&broken), &preferences), true, at)
            .started,
        0
    );
    let repaired = prepared(frame(1, at, bank(&[0, 255])), &preferences);
    assert_eq!(
        gate.tick(view(&owner, Some(&repaired), &preferences), true, at)
            .started,
        0
    );
    let loading = prepared(frame(2, at, bank(&[0, 255])), &preferences);
    let mut v = view(&owner, Some(&loading), &preferences);
    v.playable = false;
    assert_eq!(gate.tick(v, true, at).started, 0);
    assert_eq!(
        gate.tick(view(&owner, Some(&loading), &preferences), true, at)
            .started,
        0
    );
    assert_eq!(gate.sequence, 2);
}

#[test]
fn driver_failure_is_latched_across_settings_changes_retirement_and_rebind() {
    let owner = attachment();
    let at = Instant::now();
    let preferences = SharedPreferences::default();
    let ready = prepared(frame(1, at, bank(&[0, 255])), &preferences);
    let mut gate = delivery();
    gate.sink.fail = true;
    assert!(
        gate.tick(view(&owner, Some(&ready), &preferences), true, at)
            .device_failed
    );
    gate.clear();
    gate.sink.fail = false;
    let new_owner = attachment();
    let now = Instant::now();
    let next = prepared(frame(1, now, bank(&[0, 255])), &preferences);
    assert!(
        gate.tick(view(&new_owner, Some(&next), &preferences), true, now)
            .device_failed
    );
    assert_eq!(gate.sink.polls, 2);
    assert!(gate.sink.played.is_empty());
}
