//! Canonical worker-to-device boundary. No legacy Frame, wire revision,
//! HWND resource reads, global device mixer change or LOAD permission.
use super::{
    audio_gain::GainCache, combat_animation::NativeAudioGroup, combat_audio::SoundBank,
    preferences::Snapshot, world_audio::AudioFrame,
};
use anyhow::Result;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

/// Unique while any publication/gate retains it; never derived from a world
/// or object ID. Even clear/re-enter of the same world creates a new token.
pub(crate) struct Attachment {
    started: Instant,
    preferences: Snapshot,
}
impl Attachment {
    pub(crate) fn new(preferences: Snapshot) -> Arc<Self> {
        Arc::new(Self {
            started: Instant::now(),
            preferences,
        })
    }
}
pub(crate) struct PreparedAudio {
    pub frame: Option<AudioFrame>,
    pub preferences: Snapshot,
    pub gain: Option<u8>,
    pub notice: Option<&'static str>,
}
impl PreparedAudio {
    pub(crate) fn empty(preferences: Snapshot) -> Self {
        Self {
            frame: None,
            preferences,
            gain: None,
            notice: None,
        }
    }
    pub(crate) fn prepare(
        frame: Option<AudioFrame>,
        preferences: Snapshot,
        cache: &mut GainCache,
        cancel: &AtomicBool,
    ) -> Self {
        let Some(original) = frame else {
            return Self::empty(preferences);
        };
        let gain = preferences.values.audible_gain(true);
        match cache.apply(original.clone(), gain, cancel) {
            Ok(frame) => Self {
                frame: Some(frame),
                preferences,
                gain: Some(gain),
                notice: None,
            },
            Err(_) => Self {
                frame: Some(original),
                preferences,
                gain: None,
                notice: Some("音效音量轉換失敗，本批已靜音；遊戲仍可操作。"),
            },
        }
    }
}
pub(crate) struct View<'a> {
    pub attachment: Option<&'a Arc<Attachment>>,
    pub publication: Option<(&'a Arc<Attachment>, &'a PreparedAudio)>,
    pub preferences: Snapshot,
    pub playable: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlaybackStatus {
    pub started: usize,
    pub device_failed: bool,
    pub notice: Option<&'static str>,
}
trait Sink {
    fn poll(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn play(&mut self, bank: &SoundBank, group: &NativeAudioGroup, random: u32) -> Result<bool>;
}
struct Delivery<S: Sink> {
    sink: S,
    attachment: Option<Arc<Attachment>>,
    sequence: u64,
    settings: Option<(Snapshot, u8)>,
    not_before: Option<Instant>,
    sampled: Option<Instant>,
    failed: bool,
    random: u32,
}
impl<S: Sink> Delivery<S> {
    fn new(sink: S, random: u32) -> Self {
        Self {
            sink,
            attachment: None,
            sequence: 0,
            settings: None,
            not_before: None,
            sampled: None,
            failed: false,
            random: random | 1,
        }
    }
    fn stop_voices(&mut self) {
        if self.sink.stop().is_err() {
            self.failed = true;
        }
    }
    fn clear(&mut self) {
        if self.attachment.is_some() {
            self.stop_voices();
        }
        self.attachment = None;
        self.sequence = 0;
        self.settings = None;
        self.not_before = None;
    }
    fn tick(&mut self, view: View<'_>, foreground: bool, now: Instant) -> PlaybackStatus {
        // One driver query pass per UI tick, even unfocused, muted or detached.
        if self.sink.poll().is_err() {
            self.failed = true;
        }
        let mut result = PlaybackStatus::default();
        if self.sampled.is_some_and(|last| now < last) {
            self.stop_voices();
            result.notice = Some("音效時間倒退，本批已靜音。");
            result.device_failed = self.failed;
            return result;
        }
        self.sampled = Some(now);
        let Some(attachment) = view.attachment else {
            self.clear();
            result.device_failed = self.failed;
            return result;
        };
        if self
            .attachment
            .as_ref()
            .is_none_or(|old| !Arc::ptr_eq(old, attachment))
        {
            self.stop_voices();
            self.attachment = Some(attachment.clone());
            self.sequence = 0;
            self.settings = None;
            self.not_before = Some(if attachment.preferences == view.preferences {
                attachment.started
            } else {
                now
            });
        }
        let gain = view.preferences.values.audible_gain(foreground);
        let settings = (view.preferences, gain);
        if self.settings.is_some_and(|previous| previous != settings) {
            self.stop_voices();
            // Drop queued pre-change cues, including mute->unmute ABA and a
            // focus transition while the UI could not sample its mailbox.
            self.not_before = Some(now);
        }
        self.settings = Some(settings);
        if let Some((_, prepared)) = view
            .publication
            .filter(|(owner, _)| Arc::ptr_eq(owner, attachment))
        {
            result.notice = prepared.notice;
            if let Some(frame) = &prepared.frame {
                let mut previous = 0;
                let valid = frame.events.len() <= 256
                    && frame.events.iter().all(|e| {
                        let good = e.sequence > previous && e.created <= now;
                        previous = e.sequence;
                        good
                    });
                if !valid {
                    self.sequence = self
                        .sequence
                        .max(frame.events.iter().map(|e| e.sequence).max().unwrap_or(0));
                    result.notice = Some("音效批次順序或時間無效，本批已靜音。");
                } else {
                    for event in frame.events.iter() {
                        if event.sequence <= self.sequence {
                            continue;
                        }
                        self.sequence = event.sequence;
                        if self.failed
                            || !view.playable
                            || gain == 0
                            || prepared.preferences != view.preferences
                            || prepared.gain != Some(gain)
                            || self.not_before.is_some_and(|start| event.created < start)
                            || now.duration_since(event.created) > Duration::from_millis(500)
                        {
                            continue;
                        }
                        self.random ^= self.random << 13;
                        self.random ^= self.random >> 17;
                        self.random ^= self.random << 5;
                        match self.sink.play(
                            event.bank.as_deref().unwrap_or(&frame.bank),
                            &event.group,
                            self.random,
                        ) {
                            Ok(true) => result.started += 1,
                            Ok(false) => {}
                            Err(_) => {
                                self.failed = true;
                                self.stop_voices();
                            }
                        }
                    }
                }
            }
        }
        result.device_failed = self.failed;
        if self.failed {
            result.notice = Some("音效裝置發生錯誤，此視窗已停用音效；遊戲仍可操作。");
        }
        result
    }
}

#[cfg(windows)]
impl Sink for super::combat_audio::CombatVoicePool {
    fn poll(&mut self) -> Result<()> {
        self.poll()?;
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        self.stop_all()?;
        Ok(())
    }
    fn play(&mut self, bank: &SoundBank, group: &NativeAudioGroup, random: u32) -> Result<bool> {
        Ok(matches!(
            self.play(bank, group, random)?,
            super::combat_audio::PlayOutcome::Started { .. }
        ))
    }
}
/// Keep one per canonical game window. A fault remains latched across clear,
/// reload and reattachment; never create a new pool to retry a bad driver.
#[cfg(windows)]
pub struct BattleAudioOutput {
    delivery: Delivery<super::combat_audio::CombatVoicePool>,
}
#[cfg(windows)]
impl BattleAudioOutput {
    /// Allocates a fixed eight-voice pool, but does not open an audio device.
    pub fn new() -> Result<Self> {
        let random = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        Ok(Self {
            delivery: Delivery::new(super::combat_audio::CombatVoicePool::new(8)?, random),
        })
    }
    /// Call after runtime.poll on every UI tick, not only WM_PAINT or focus.
    pub fn tick(
        &mut self,
        runtime: &super::battle_runtime::BattleRuntime,
        foreground: bool,
        now: Instant,
    ) -> PlaybackStatus {
        self.delivery.tick(runtime.audio_view(), foreground, now)
    }
    /// Stop voices without forgetting already consumed cues. Clear/shutdown
    /// the runtime to retire an attachment; stop alone is not a mute setting.
    pub fn stop(&mut self) {
        self.delivery.stop_voices();
    }
    pub fn is_quiescent(&self) -> bool {
        self.delivery.sink.status().retained == 0
    }
}

#[cfg(test)]
#[path = "battle_playback_tests.rs"]
pub(crate) mod tests;
