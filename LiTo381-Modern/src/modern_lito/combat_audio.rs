//! Shared-only battle cue preparation and a fixed-capacity native voice pool.
//!
//! `SoundBank::load_shared` belongs on the owned resource worker, never on a UI
//! or network thread. All candidate WAVs must validate before the immutable bank
//! is published (bank payload <= 64 MiB, plus <= 16 MiB transient WAV conversion
//! storage and bounded index/chunk metadata). The pool only consumes prepared PCM: no filenames, decoding,
//! combat RNG, hit processing, LT fallback, or distance (`^`) approximation.
//!
//! Device lifecycle (issue #341). Keep one pool for the window lifetime; its
//! fixed voice objects are never replaced, so recreating pools would evade the
//! bounds below.
//! - No output device (open refused, nothing handed to the driver): not a
//!   fault. The cue is dropped, the slot stays free and the next open is
//!   attempted no sooner than `DEVICE_RETRY_MILLIS` later.
//! - Any other device failure faults the pool: every voice is stopped and
//!   cleanup is retried without allocating replacements. Once the driver owns
//!   nothing and `FAULT_RECOVERY_MILLIS` passed without a new failure, `poll`
//!   re-enables the pool, at most `MAX_FAULT_RECOVERIES` times; after that the
//!   fault is permanent.
//! - Dropped cues are never queued or replayed.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};

use super::audio::{parse_pcm_wav, PcmFormat, WavePadding, MAX_WAVE_BYTES};
use super::combat_animation::{NativeAudioGroup, NativeAudioMode};
use super::resources::{read_shared_sound_with_limit, SharedSoundKey};

pub const MAX_BANK_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_BANK_SOUNDS: usize = 512;
pub const MAX_BANK_GROUPS: usize = 4096;
pub const MAX_VOICES: usize = 16;
/// Longest admitted cue. Every original shared `Sound/<1..9999>.wav` is at
/// most 22.3 s (`Sound/361.wav`); a longer or near-zero-rate WAV would hold a
/// fixed voice for an unbounded time and is refused before publication.
pub const MAX_PLAYBACK_MILLIS: u64 = 60_000;
/// Driver latency allowed beyond the PCM length before a voice that still
/// reports "playing" is reset by the pool (never restarted or replaced).
pub const VOICE_OVERRUN_GRACE_MILLIS: u64 = 5_000;
/// After an open that found no usable output device, cues are dropped without
/// touching the driver for this long. Bounds a 256-cue batch on a machine
/// without speakers to one failed open, and lets a plugged-in device be picked
/// up by the first cue after the interval.
pub const DEVICE_RETRY_MILLIS: u64 = 2_000;
/// Quiet time (no new device error, nothing owned by the driver) before a
/// faulted pool is re-enabled by `poll`.
pub const FAULT_RECOVERY_MILLIS: u64 = 5_000;
/// Re-enables per pool. A driver that keeps failing ends permanently faulted.
pub const MAX_FAULT_RECOVERIES: u8 = 8;

#[derive(Clone, Copy, Debug)]
pub struct BankLimits {
    pub max_bytes: usize,
    pub max_sounds: usize,
    pub max_groups: usize,
}
impl Default for BankLimits {
    fn default() -> Self {
        Self {
            max_bytes: 32 * 1024 * 1024,
            max_sounds: MAX_BANK_SOUNDS,
            max_groups: MAX_BANK_GROUPS,
        }
    }
}
impl BankLimits {
    fn validate(self) -> Result<Self> {
        ensure!(
            (1..=MAX_BANK_BYTES).contains(&self.max_bytes)
                && (1..=MAX_BANK_SOUNDS).contains(&self.max_sounds)
                && (1..=MAX_BANK_GROUPS).contains(&self.max_groups),
            "invalid bounded shared sound bank limits"
        );
        Ok(self)
    }
}

/// One immutable, already-qualified WAV. Its original bytes and PCM range are
/// the same generation; playback retains a cheap clone of this individual file,
/// not the whole bank. Construction may allocate/parse and is worker-only.
#[derive(Clone)]
pub struct PreparedSound {
    sound_id: u32,
    wav: Arc<[u8]>,
    pcm: Range<usize>,
    format: PcmFormat,
    padding: WavePadding,
    sha256: [u8; 32],
    /// Optional playback-only gain; source WAV/hash remain unchanged.
    gained_pcm: Option<Arc<[u8]>>,
    /// Playback-only native-range volume/pan. None preserves the source layout.
    /// The new backend uses linear balance, not native middleware/DirectSound.
    spatial: Option<(u8, u8)>,
}
impl std::fmt::Debug for PreparedSound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedSound")
            .field("sound_id", &self.sound_id)
            .field("file_bytes", &self.file_bytes())
            .field("pcm_bytes", &self.pcm_bytes().len())
            .field("format", &self.format)
            .finish_non_exhaustive()
    }
}
impl PreparedSound {
    pub fn from_wav(sound_id: u32, bytes: Vec<u8>) -> Result<Self> {
        ensure!(
            (1..10_000).contains(&sound_id),
            "sound is not a shared native ID"
        );
        ensure!(bytes.len() <= MAX_WAVE_BYTES, "shared WAV exceeds 16 MiB");
        let wave = parse_pcm_wav(&bytes).context("qualify shared battle PCM")?;
        ensure!(!wave.pcm_bytes().is_empty(), "shared battle PCM is empty");
        let pcm = wave
            .chunks()
            .iter()
            .find(|chunk| chunk.id == *b"data")
            .context("qualified PCM has no data range")?
            .data_range
            .clone();
        ensure!(
            bytes.get(pcm.clone()) == Some(wave.pcm_bytes()),
            "PCM range mismatch"
        );
        let format = wave.format();
        ensure!(
            playback_millis(wave.pcm_bytes().len(), format) <= MAX_PLAYBACK_MILLIS,
            "shared battle PCM exceeds the bounded playback length"
        );
        let padding = wave.padding();
        let sha256 = Sha256::digest(&bytes).into();
        Ok(Self {
            sound_id,
            wav: Arc::from(bytes.into_boxed_slice()),
            pcm,
            format,
            padding,
            sha256,
            gained_pcm: None,
            spatial: None,
        })
    }
    pub fn sound_id(&self) -> u32 {
        self.sound_id
    }
    pub fn format(&self) -> PcmFormat {
        let mut format = self.format;
        if self.spatial.is_some() && format.channels == 1 {
            format.channels = 2;
            format.byte_rate *= 2;
            format.block_align *= 2;
        }
        format
    }
    pub fn pcm_bytes(&self) -> &[u8] {
        self.gained_pcm
            .as_deref()
            .unwrap_or(&self.wav[self.pcm.clone()])
    }
    /// Worker-only, integer attenuation of the original qualified samples.
    /// Always scale the source, never compound previous attenuation; no mixer API.
    /// Spatial attenuation is composed with (not erased by) the user volume.
    pub(crate) fn with_gain(&self, percent: u8) -> Result<Self> {
        ensure!(percent <= 100, "invalid PCM gain");
        let mut gained = self.clone();
        gained.gained_pcm = None;
        if let Some((volume, pan)) = self.spatial {
            let source = &self.wav[self.pcm.clone()];
            let channels = usize::from(self.format.channels);
            let width = usize::from(self.format.bits_per_sample / 8);
            let size = source
                .len()
                .checked_mul(2 / channels)
                .context("spatial PCM overflow")?;
            ensure!(size <= MAX_WAVE_BYTES, "spatial PCM exceeds bound");
            let mut pcm = Vec::new();
            pcm.try_reserve_exact(size)?;
            // Preserve an existing stereo source; mono is duplicated before
            // balance. i64 covers full-scale signed PCM and all gain factors.
            let balance = [(128 - i64::from(pan)).min(64), i64::from(pan).min(64)];
            for frame in source.chunks_exact(channels * width) {
                for (channel, balance) in balance.into_iter().enumerate() {
                    let offset = if channels == 1 { 0 } else { channel * width };
                    let value = if width == 1 {
                        i64::from(frame[offset]) - 128
                    } else {
                        i64::from(i16::from_le_bytes([frame[offset], frame[offset + 1]]))
                    };
                    let value =
                        value * i64::from(volume) * i64::from(percent) * balance / (127 * 100 * 64);
                    if width == 1 {
                        pcm.push((value + 128) as u8);
                    } else {
                        pcm.extend_from_slice(&(value as i16).to_le_bytes());
                    }
                }
            }
            ensure!(pcm.len() == size, "spatial PCM alignment changed");
            gained.gained_pcm = Some(pcm.into());
            return Ok(gained);
        }
        if percent == 100 {
            return Ok(gained);
        }
        let source = &self.wav[self.pcm.clone()];
        let mut pcm = Vec::new();
        pcm.try_reserve_exact(source.len())?;
        match self.format.sample_format {
            super::audio::PcmSampleFormat::Unsigned8 => {
                pcm.extend(
                    source
                        .iter()
                        .map(|&x| (((i32::from(x) - 128) * i32::from(percent)) / 100 + 128) as u8),
                );
            }
            super::audio::PcmSampleFormat::Signed16 => {
                for sample in source.chunks_exact(2) {
                    let x = i32::from(i16::from_le_bytes([sample[0], sample[1]]));
                    pcm.extend_from_slice(&((x * i32::from(percent) / 100) as i16).to_le_bytes());
                }
            }
        }
        ensure!(pcm.len() == source.len(), "PCM alignment changed");
        gained.gained_pcm = Some(pcm.into());
        Ok(gained)
    }
    pub(crate) fn spatial_gain(&self) -> Option<(u8, u8)> {
        self.spatial
    }
    pub(crate) fn with_spatial(&self, volume: u8, pan: u8) -> Result<Self> {
        ensure!(volume <= 127 && pan <= 128, "invalid spatial PCM gain");
        let mut sound = self.clone();
        sound.spatial = Some((volume, pan));
        sound.with_gain(100)
    }
    pub fn file_bytes(&self) -> usize {
        self.wav.len()
    }
    /// Whole playback length, rounded up; 1..=MAX_PLAYBACK_MILLIS. Gain and
    /// spatial variants keep the source frame count and therefore this value.
    pub fn playback_millis(&self) -> u64 {
        playback_millis(self.pcm_bytes().len(), self.format())
    }
    pub fn padding(&self) -> WavePadding {
        self.padding
    }
    pub fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
}

fn playback_millis(pcm_bytes: usize, format: PcmFormat) -> u64 {
    // The strict parser guarantees byte_rate = sample_rate * block_align >= 1.
    (pcm_bytes as u64 * 1000).div_ceil(u64::from(format.byte_rate))
}

#[derive(Debug)]
pub struct SoundBank {
    sounds: BTreeMap<u32, PreparedSound>,
    file_bytes: usize,
    pcm_bytes: usize,
}
impl SoundBank {
    /// A playback-only subset, still retaining qualified source metadata. This
    /// constructor never reads a file and cannot invent a missing sound.
    pub(crate) fn playback_subset(sounds: BTreeMap<u32, PreparedSound>) -> Result<Self> {
        ensure!(
            sounds.len() <= MAX_BANK_SOUNDS,
            "playback bank count exceeded"
        );
        let file_bytes: usize = sounds.values().map(PreparedSound::file_bytes).sum();
        let pcm_bytes: usize = sounds.values().map(|s| s.pcm_bytes().len()).sum();
        ensure!(
            file_bytes <= MAX_BANK_BYTES && pcm_bytes <= MAX_BANK_BYTES,
            "playback bank size exceeded"
        );
        Ok(Self {
            sounds,
            file_bytes,
            pcm_bytes,
        })
    }
    /// Load every nonzero weighted alternative once. Unknown/missing/ambiguous
    /// files, unsupported markers, cancellation and budget overflow fail the
    /// entire bank, without publishing a partially playable subset. Cancellation
    /// is checked between bounded reads; it cannot interrupt a filesystem call.
    pub fn load_shared(
        root: impl AsRef<Path>,
        groups: &[NativeAudioGroup],
        limits: BankLimits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let root = root.as_ref();
        Self::load_with(groups, limits, cancel, |id, limit| {
            read_shared_sound_with_limit(root, SharedSoundKey { sound_id: id }, limit as u64)
                .with_context(|| format!("read shared Sound/{id}.wav"))
        })
    }
    fn load_with(
        groups: &[NativeAudioGroup],
        limits: BankLimits,
        cancel: &AtomicBool,
        mut read: impl FnMut(u32, usize) -> Result<Vec<u8>>,
    ) -> Result<Self> {
        let limits = limits.validate()?;
        active(cancel)?;
        ensure!(
            groups.len() <= limits.max_groups,
            "shared sound group budget exceeded"
        );
        let mut ids = BTreeSet::new();
        for group in groups {
            active(cancel)?;
            validate_group(group)?;
            for &id in group.candidates() {
                if id != 0 {
                    ids.insert(id);
                }
            }
            ensure!(
                ids.len() <= limits.max_sounds,
                "shared sound ID budget exceeded"
            );
        }
        let mut bank = Self {
            sounds: BTreeMap::new(),
            file_bytes: 0,
            pcm_bytes: 0,
        };
        for id in ids {
            active(cancel)?;
            let remaining = limits
                .max_bytes
                .checked_sub(bank.file_bytes)
                .context("shared sound bank byte accounting overflow")?;
            ensure!(remaining > 0, "shared sound bank byte budget exhausted");
            let limit = remaining.min(MAX_WAVE_BYTES);
            let bytes = read(id, limit)?;
            active(cancel)?;
            ensure!(
                bytes.len() <= limit,
                "shared sound reader exceeded its byte budget"
            );
            let sound = PreparedSound::from_wav(id, bytes)?;
            active(cancel)?;
            bank.file_bytes = bank
                .file_bytes
                .checked_add(sound.file_bytes())
                .context("shared sound bank byte accounting overflow")?;
            bank.pcm_bytes = bank
                .pcm_bytes
                .checked_add(sound.pcm_bytes().len())
                .context("shared sound PCM accounting overflow")?;
            bank.sounds.insert(id, sound);
        }
        active(cancel)?;
        Ok(bank)
    }
    pub fn get(&self, sound_id: u32) -> Option<&PreparedSound> {
        self.sounds.get(&sound_id)
    }
    pub fn len(&self) -> usize {
        self.sounds.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sounds.is_empty()
    }
    pub fn file_bytes(&self) -> usize {
        self.file_bytes
    }
    pub fn pcm_bytes(&self) -> usize {
        self.pcm_bytes
    }
    pub fn ids(&self) -> impl ExactSizeIterator<Item = u32> + '_ {
        self.sounds.keys().copied()
    }
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "shared sound bank load cancelled"
    );
    Ok(())
}
fn validate_group(group: &NativeAudioGroup) -> Result<()> {
    ensure!(
        group.mode != NativeAudioMode::LegacyCaretDistanceMode,
        "native caret audio is unsupported"
    );
    ensure!(
        (1..=3).contains(&group.candidates().len()),
        "invalid native audio group size"
    );
    ensure!(
        group.candidates().iter().all(|id| *id < 10_000),
        "LT audio is not shared battle audio"
    );
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayOutcome {
    Started {
        sound_id: u32,
        slot: usize,
    },
    Suppressed {
        sound_id: u32,
    },
    Silent,
    /// Deliberately drops this cue, never steals or restarts an active voice.
    VoiceLimit,
    /// No usable output device (none present, audio service stopped, or held
    /// exclusively). The cue is dropped; nothing is retained or replayed.
    DeviceUnavailable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolStatus {
    pub active: usize,
    /// Slots that may still own a native header/buffer, including failed starts.
    pub retained: usize,
    pub faulted: bool,
    /// The most recent open found no usable output device. Cleared by the
    /// next successful start; a state of the host, not a fault of the pool.
    pub device_unavailable: bool,
}

#[cfg(windows)]
mod output {
    use super::super::audio_output::{AudioOutputError, PcmVoice, VoiceStatus};
    use super::*;
    use std::time::{Duration, Instant};

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum PoolError {
        InvalidCapacity,
        UnsupportedGroup,
        MissingSound(u32),
        /// The first device failure is preserved on every later operation.
        Device(AudioOutputError),
    }
    impl std::fmt::Display for PoolError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::InvalidCapacity => {
                    write!(f, "battle sound voice capacity must be 1..={MAX_VOICES}")
                }
                Self::UnsupportedGroup => write!(f, "unsupported shared battle sound group"),
                Self::MissingSound(id) => {
                    write!(f, "Sound/{id}.wav was not qualified in the sound bank")
                }
                Self::Device(error) => {
                    write!(f, "battle sound disabled after device failure: {error}")
                }
            }
        }
    }
    impl std::error::Error for PoolError {}

    trait OutputVoice {
        fn start(&mut self, sound: &PreparedSound) -> Result<(), AudioOutputError>;
        fn poll(&mut self) -> Result<VoiceStatus, AudioOutputError>;
        fn stop(&mut self) -> Result<(), AudioOutputError>;
        /// The driver owns nothing of this voice. No driver call.
        fn is_idle(&self) -> bool;
    }
    impl OutputVoice for PcmVoice {
        fn is_idle(&self) -> bool {
            PcmVoice::is_idle(self)
        }
        fn start(&mut self, sound: &PreparedSound) -> Result<(), AudioOutputError> {
            self.start_prepared(sound)
        }
        fn poll(&mut self) -> Result<VoiceStatus, AudioOutputError> {
            PcmVoice::poll(self)
        }
        fn stop(&mut self) -> Result<(), AudioOutputError> {
            PcmVoice::stop(self)
        }
    }
    struct Slot<V> {
        voice: V,
        sound_id: Option<u32>,
        playing: bool,
        /// Start time + bounded PCM length + grace. Only set by a successful
        /// start; a voice still "playing" past it is reset by `poll`.
        deadline: Option<Instant>,
    }
    struct Pool<V: OutputVoice> {
        slots: Vec<Slot<V>>,
        fault: Option<AudioOutputError>,
        /// Monotonic; replaced only by deterministic tests.
        clock: fn() -> Instant,
        expired: u64,
        /// Time of the latest device failure while faulted (latch or a
        /// failed cleanup retry). Recovery counts quiet time from here.
        fault_at: Option<Instant>,
        recoveries: u8,
        /// Set by an open that found no device; cues before it are dropped
        /// without a driver call. Kept (expired) until a start succeeds.
        retry_open_at: Option<Instant>,
        unavailable_cues: u64,
    }
    impl<V: OutputVoice> Pool<V> {
        fn new(capacity: usize, mut create: impl FnMut() -> V) -> Result<Self, PoolError> {
            if !(1..=MAX_VOICES).contains(&capacity) {
                return Err(PoolError::InvalidCapacity);
            }
            Ok(Self {
                slots: (0..capacity)
                    .map(|_| Slot {
                        voice: create(),
                        sound_id: None,
                        playing: false,
                        deadline: None,
                    })
                    .collect(),
                fault: None,
                clock: Instant::now,
                expired: 0,
                fault_at: None,
                recoveries: 0,
                retry_open_at: None,
                unavailable_cues: 0,
            })
        }
        fn latch(&mut self, error: AudioOutputError) {
            if self.fault.is_none() {
                self.fault = Some(error);
            }
            self.fault_at = Some((self.clock)());
        }
        /// Re-enable a faulted pool only when the driver owns nothing, the
        /// failure has been quiet for `FAULT_RECOVERY_MILLIS`, and the bounded
        /// number of re-enables is not used up. Called from `poll` only.
        fn recover_if_quiet(&mut self) {
            let quiet = self.fault_at.is_none_or(|at| {
                (self.clock)().saturating_duration_since(at)
                    >= Duration::from_millis(FAULT_RECOVERY_MILLIS)
            });
            if self.fault.is_some()
                && quiet
                && self.recoveries < MAX_FAULT_RECOVERIES
                && self.slots.iter().all(|slot| slot.sound_id.is_none())
            {
                self.fault = None;
                self.fault_at = None;
                self.recoveries += 1;
            }
        }
        fn status(&self) -> PoolStatus {
            PoolStatus {
                active: self.slots.iter().filter(|slot| slot.playing).count(),
                retained: self
                    .slots
                    .iter()
                    .filter(|slot| slot.sound_id.is_some())
                    .count(),
                faulted: self.fault.is_some(),
                device_unavailable: self.retry_open_at.is_some(),
            }
        }
        fn play(
            &mut self,
            bank: &SoundBank,
            group: &NativeAudioGroup,
            selection: u32,
        ) -> Result<PlayOutcome, PoolError> {
            if let Some(error) = &self.fault {
                return Err(PoolError::Device(error.clone()));
            }
            validate_group(group).map_err(|_| PoolError::UnsupportedGroup)?;
            // Validate the whole weighted group, not only the chosen branch.
            for &id in group.candidates() {
                if id != 0 && bank.get(id).is_none() {
                    return Err(PoolError::MissingSound(id));
                }
            }
            let Some(id) = group.selected(selection) else {
                return Ok(PlayOutcome::Silent);
            };
            self.play_prepared(bank.get(id).expect("all candidates qualified"),group.mode)
        }
        fn play_prepared(&mut self,sound:&PreparedSound,mode:NativeAudioMode)->Result<PlayOutcome,PoolError> {
            if let Some(error)=&self.fault {return Err(PoolError::Device(error.clone()));}
            let id=sound.sound_id();
            if mode == NativeAudioMode::SuppressWhileSameIdPlaying
                && self
                    .slots
                    .iter()
                    .any(|slot| slot.playing && slot.sound_id == Some(id))
            {
                return Ok(PlayOutcome::Suppressed { sound_id: id });
            }
            let now = (self.clock)();
            // No device a moment ago: drop the cue without another open.
            // Voices that are already playing are left alone.
            if self.retry_open_at.is_some_and(|at| now < at) {
                self.unavailable_cues = self.unavailable_cues.saturating_add(1);
                return Ok(PlayOutcome::DeviceUnavailable);
            }
            let Some((index, slot)) = self
                .slots
                .iter_mut()
                .enumerate()
                .find(|(_, slot)| slot.sound_id.is_none())
            else {
                return Ok(PlayOutcome::VoiceLimit);
            };
            // A failed start may still retain native-owned memory. Never clear
            // its identity or replace this fixed voice until stop succeeds.
            slot.sound_id = Some(id);
            match slot
                .voice
                .start(sound)
            {
                Ok(()) => {
                    slot.playing = true;
                    slot.deadline = now.checked_add(Duration::from_millis(
                        sound.playback_millis() + VOICE_OVERRUN_GRACE_MILLIS,
                    ));
                    self.retry_open_at = None;
                    Ok(PlayOutcome::Started {
                        sound_id: id,
                        slot: index,
                    })
                }
                // Nothing reached the driver and the host says there is no
                // usable device: free the slot, back off, do not fault.
                Err(error) if error.device_unavailable() && slot.voice.is_idle() => {
                    slot.sound_id = None;
                    self.retry_open_at =
                        now.checked_add(Duration::from_millis(DEVICE_RETRY_MILLIS));
                    self.unavailable_cues = self.unavailable_cues.saturating_add(1);
                    Ok(PlayOutcome::DeviceUnavailable)
                }
                Err(error) => {
                    self.latch(error.clone());
                    self.cleanup_once();
                    Err(PoolError::Device(error))
                }
            }
        }
        fn poll(&mut self) -> Result<PoolStatus, PoolError> {
            if self.fault.is_some() {
                self.cleanup_once();
                self.recover_if_quiet();
                if let Some(error) = &self.fault {
                    return Err(PoolError::Device(error.clone()));
                }
            }
            let now = (self.clock)();
            for slot in &mut self.slots {
                if slot.sound_id.is_none() {
                    continue;
                }
                let status = match slot.voice.poll() {
                    // A finite cue never owns its voice past its own length:
                    // a driver that keeps reporting "playing" (stalled or
                    // looping device) is reset once, without a restart.
                    Ok(VoiceStatus::Playing) if slot.deadline.is_some_and(|d| now >= d) => {
                        slot.voice.stop().map(|()| {
                            self.expired = self.expired.saturating_add(1);
                            VoiceStatus::Finished
                        })
                    }
                    other => other,
                };
                match status {
                    Ok(VoiceStatus::Playing) => slot.playing = true,
                    Ok(VoiceStatus::Idle | VoiceStatus::Finished) => {
                        slot.sound_id = None;
                        slot.playing = false;
                        slot.deadline = None;
                    }
                    Err(error) => {
                        self.fault = Some(error);
                        self.fault_at = Some(now);
                        break;
                    }
                }
            }
            if self.fault.is_some() {
                self.cleanup_once();
                Err(PoolError::Device(self.fault.as_ref().unwrap().clone()))
            } else {
                Ok(self.status())
            }
        }
        fn cleanup_once(&mut self) {
            let mut failed = None;
            for slot in &mut self.slots {
                if slot.sound_id.is_none() {
                    continue;
                }
                match slot.voice.stop() {
                    Ok(()) => {
                        slot.sound_id = None;
                        slot.playing = false;
                        slot.deadline = None;
                    }
                    Err(error) => {
                        failed.get_or_insert(error);
                    }
                }
            }
            // A failed stop (re)starts the quiet period before recovery.
            if let Some(error) = failed {
                self.latch(error);
            }
        }
        fn stop_all(&mut self) -> Result<(), PoolError> {
            self.cleanup_once();
            if let Some(error) = &self.fault {
                Err(PoolError::Device(error.clone()))
            } else {
                Ok(())
            }
        }
    }
    impl<V: OutputVoice> Drop for Pool<V> {
        fn drop(&mut self) {
            self.cleanup_once();
        }
    }

    /// Fixed main-thread pool. `new` never opens a device. `poll` performs at
    /// most one query pass and one cleanup pass, with no file IO, decoding,
    /// sleeps or wait loop. WinMM calls themselves remain driver-dependent.
    /// The caller must poll once before each tick's cue batch. `play` never
    /// polls the driver: a just-finished voice conservatively stays occupied
    /// until the next poll, bounding a 256-cue batch to one native query pass.
    /// Keep polling independently of focus/WM_PAINT; a fault report must mute
    /// the window's audio, never trigger a replacement-pool retry loop. The
    /// only re-enable is this pool's own bounded recovery inside `poll`
    /// (`Ok` again after an `Err`); a missing device is not reported as a
    /// fault at all (`PlayOutcome::DeviceUnavailable`).
    pub struct CombatVoicePool {
        inner: Pool<PcmVoice>,
    }
    impl CombatVoicePool {
        pub fn new(capacity: usize) -> Result<Self, PoolError> {
            Ok(Self {
                inner: Pool::new(capacity, PcmVoice::new)?,
            })
        }
        pub fn play(
            &mut self,
            bank: &SoundBank,
            group: &NativeAudioGroup,
            selection: u32,
        ) -> Result<PlayOutcome, PoolError> {
            self.inner.play(bank, group, selection)
        }
        /// Preserve the legacy caller's overlapping single-WAV delivery, using
        /// the same fixed slots and same-ID registry as native classified groups.
        /// The prepared file identity is retained; no namespace fallback occurs.
        pub fn play_legacy(&mut self,sound:&PreparedSound)->Result<PlayOutcome,PoolError> {
            self.inner.play_prepared(sound,NativeAudioMode::AllowOverlappingVoices)
        }
        pub fn poll(&mut self) -> Result<PoolStatus, PoolError> {
            self.inner.poll()
        }
        pub fn status(&self) -> PoolStatus {
            self.inner.status()
        }
        /// Voices reset by `poll` because the driver still reported them
        /// playing after PCM length + `VOICE_OVERRUN_GRACE_MILLIS`. Zero on a
        /// healthy device; diagnostics only, never a reason to replay a cue.
        pub fn expired_voices(&self) -> u64 {
            self.inner.expired
        }
        /// Cues dropped because no usable output device existed. Diagnostics
        /// only; dropped cues are never replayed.
        pub fn unavailable_cues(&self) -> u64 {
            self.inner.unavailable_cues
        }
        /// Times `poll` re-enabled this pool after a device fault
        /// (0..=`MAX_FAULT_RECOVERIES`).
        pub fn fault_recoveries(&self) -> u8 {
            self.inner.recoveries
        }
        /// Healthy pools can be reused after a match (scene switch, exit).
        /// A device fault stays latched here even when this cleanup succeeds;
        /// only `poll` re-enables the pool, after the bounded quiet period.
        pub fn stop_all(&mut self) -> Result<(), PoolError> {
            self.inner.stop_all()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::modern_lito::audio_output::DriverError;
        use std::cell::RefCell;
        use std::rc::Rc;
        #[derive(Default)]
        struct Trace {
            starts: usize,
            stops: usize,
            polls: usize,
            playing: bool,
            start_fail: bool,
            poll_fail: bool,
            stop_fail: bool,
            retained: Option<PreparedSound>,
            /// Issue #341: the open is refused with this code; like WinMM,
            /// nothing is handed to the driver.
            no_device: Option<u32>,
            /// Replaces the generic error of a `start_fail` (buffer retained).
            start_error: Option<AudioOutputError>,
        }
        struct Fake(Rc<RefCell<Trace>>);
        fn error() -> AudioOutputError {
            AudioOutputError::Device(DriverError {
                operation: "fake",
                code: 5,
            })
        }
        impl OutputVoice for Fake {
            fn start(&mut self, sound: &PreparedSound) -> Result<(), AudioOutputError> {
                let mut trace = self.0.borrow_mut();
                trace.starts += 1;
                if let Some(code) = trace.no_device {
                    return Err(AudioOutputError::Device(DriverError {
                        operation: "waveOutOpen",
                        code,
                    }));
                }
                trace.retained = Some(sound.clone());
                if trace.start_fail {
                    Err(trace.start_error.clone().unwrap_or_else(error))
                } else {
                    trace.playing = true;
                    Ok(())
                }
            }
            fn poll(&mut self) -> Result<VoiceStatus, AudioOutputError> {
                let mut trace = self.0.borrow_mut();
                trace.polls += 1;
                if trace.poll_fail {
                    Err(error())
                } else if trace.playing {
                    Ok(VoiceStatus::Playing)
                } else {
                    trace.retained = None;
                    Ok(VoiceStatus::Finished)
                }
            }
            fn is_idle(&self) -> bool {
                self.0.borrow().retained.is_none()
            }
            fn stop(&mut self) -> Result<(), AudioOutputError> {
                let mut trace = self.0.borrow_mut();
                trace.stops += 1;
                if trace.stop_fail {
                    Err(error())
                } else {
                    trace.playing = false;
                    trace.retained = None;
                    Ok(())
                }
            }
        }
        fn pool(cap: usize) -> (Pool<Fake>, Vec<Rc<RefCell<Trace>>>) {
            let mut traces = Vec::new();
            let pool = Pool::new(cap, || {
                let trace = Rc::new(RefCell::new(Trace::default()));
                traces.push(trace.clone());
                Fake(trace)
            })
            .unwrap();
            (pool, traces)
        }
        fn bank() -> SoundBank {
            super::super::tests::bank(&[super::super::tests::group(b'<', &[1, 2])])
        }
        fn group(marker: u8, ids: &[u32]) -> NativeAudioGroup {
            super::super::tests::group(marker, ids)
        }
        #[test]
        fn legacy_and_native_share_same_id_guard_and_keep_namespace_validation() {
            let bank=bank();let (mut pool,traces)=pool(3);
            assert!(matches!(pool.play_prepared(bank.get(1).unwrap(),NativeAudioMode::AllowOverlappingVoices).unwrap(),PlayOutcome::Started{slot:0,..}));
            assert_eq!(pool.play(&bank,&group(b'[',&[1]),0).unwrap(),PlayOutcome::Suppressed{sound_id:1});
            assert!(matches!(pool.play_prepared(bank.get(1).unwrap(),NativeAudioMode::AllowOverlappingVoices).unwrap(),PlayOutcome::Started{slot:1,..}));
            assert!(PreparedSound::from_wav(10001,bank.get(1).unwrap().wav.to_vec()).is_err());
            assert!(matches!(pool.play_prepared(bank.get(2).unwrap(),NativeAudioMode::AllowOverlappingVoices).unwrap(),PlayOutcome::Started{sound_id:2,slot:2}));
            assert_eq!(pool.play(&bank,&group(b'[',&[2]),0).unwrap(),PlayOutcome::Suppressed{sound_id:2});
            assert_eq!(traces.iter().map(|t|t.borrow().starts).sum::<usize>(),3);
        }
        #[test]
        fn legacy_start_fault_latches_both_legacy_and_native_admission() {
            let bank=bank();let (mut pool,traces)=pool(2);traces[0].borrow_mut().start_fail=true;
            assert!(matches!(pool.play_prepared(bank.get(1).unwrap(),NativeAudioMode::AllowOverlappingVoices),Err(PoolError::Device(_))));
            assert!(matches!(pool.play(&bank,&group(b'[',&[1]),0),Err(PoolError::Device(_))));
            assert!(matches!(pool.play_prepared(bank.get(2).unwrap(),NativeAudioMode::AllowOverlappingVoices),Err(PoolError::Device(_))));
            assert_eq!(traces.iter().map(|t|t.borrow().starts).sum::<usize>(),1);
            assert!(pool.status().faulted);
        }
        #[test]
        fn overlapping_suppressing_capacity_and_finished_slot_reuse_are_exact() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            assert_eq!(
                pool.play(&bank, &group(b'<', &[1]), 0).unwrap(),
                PlayOutcome::Started {
                    sound_id: 1,
                    slot: 0
                }
            );
            assert_eq!(
                pool.play(&bank, &group(b'[', &[1]), 0).unwrap(),
                PlayOutcome::Suppressed { sound_id: 1 }
            );
            assert_eq!(
                pool.play(&bank, &group(b'<', &[1]), 0).unwrap(),
                PlayOutcome::Started {
                    sound_id: 1,
                    slot: 1
                }
            );
            assert_eq!(
                pool.play(&bank, &group(b'<', &[2]), 0).unwrap(),
                PlayOutcome::VoiceLimit
            );
            assert_eq!(traces.iter().map(|t| t.borrow().starts).sum::<usize>(), 2);
            traces[0].borrow_mut().playing = false;
            pool.poll().unwrap();
            assert_eq!(
                pool.play(&bank, &group(b'<', &[2]), 0).unwrap(),
                PlayOutcome::Started {
                    sound_id: 2,
                    slot: 0
                }
            );
            pool.stop_all().unwrap();
            assert_eq!(pool.status().retained, 0);
            pool.play(&bank, &group(b'<', &[1]), 0).unwrap(); // Healthy pool is reusable.
        }
        #[test]
        fn a_cue_batch_never_queries_driver_and_finished_slots_wait_for_next_tick() {
            let bank = bank();
            let (mut pool, traces) = pool(1);
            pool.poll().unwrap();
            pool.play(&bank, &group(b'<', &[1]), 0).unwrap();
            traces[0].borrow_mut().playing = false;
            for _ in 0..256 {
                assert_eq!(
                    pool.play(&bank, &group(b'<', &[2]), 0).unwrap(),
                    PlayOutcome::VoiceLimit
                );
                assert_eq!(
                    pool.play(&bank, &group(b'[', &[1]), 0).unwrap(),
                    PlayOutcome::Suppressed { sound_id: 1 }
                );
            }
            assert_eq!(traces[0].borrow().polls, 0);
            assert_eq!(traces[0].borrow().starts, 1);
            pool.poll().unwrap();
            assert_eq!(traces[0].borrow().polls, 1);
            assert_eq!(
                pool.play(&bank, &group(b'<', &[2]), 0).unwrap(),
                PlayOutcome::Started {
                    sound_id: 2,
                    slot: 0
                }
            );
            assert_eq!(traces[0].borrow().polls, 1);
        }
        #[test]
        fn silent_weighted_selection_and_unsupported_groups_never_start_device() {
            let bank = bank();
            let (mut pool, traces) = pool(1);
            assert_eq!(
                pool.play(&bank, &group(b'<', &[1, 0, 1]), 1).unwrap(),
                PlayOutcome::Silent
            );
            assert_eq!(
                pool.play(&bank, &group(b'^', &[1]), 0),
                Err(PoolError::UnsupportedGroup)
            );
            assert_eq!(
                pool.play(&bank, &group(b'<', &[1, 3]), 0),
                Err(PoolError::MissingSound(3))
            );
            assert_eq!(traces[0].borrow().starts, 0);
        }
        #[test]
        fn failed_start_and_persistent_cleanup_never_allocate_another_voice() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            traces[0].borrow_mut().start_fail = true;
            traces[0].borrow_mut().stop_fail = true;
            assert!(pool.play(&bank, &group(b'<', &[1]), 0).is_err());
            for _ in 0..20 {
                assert!(pool.play(&bank, &group(b'<', &[2]), 0).is_err());
                assert!(pool.poll().is_err());
            }
            assert_eq!(
                pool.status(),
                PoolStatus {
                    active: 0,
                    retained: 1,
                    faulted: true,
                    device_unavailable: false,
                }
            );
            assert_eq!(traces[0].borrow().starts, 1);
            assert_eq!(traces[1].borrow().starts, 0);
            traces[0].borrow_mut().stop_fail = false;
            assert!(pool.stop_all().is_err());
            assert_eq!(pool.status().retained, 0);
            assert!(pool.play(&bank, &group(b'<', &[2]), 0).is_err());
            assert_eq!(traces[1].borrow().starts, 0);
        }
        #[test]
        fn poll_failure_stops_all_voices_and_drop_retries_only_retained_slot() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            pool.play(&bank, &group(b'<', &[1]), 0).unwrap();
            pool.play(&bank, &group(b'<', &[2]), 0).unwrap();
            traces[0].borrow_mut().poll_fail = true;
            traces[0].borrow_mut().stop_fail = true;
            assert!(pool.poll().is_err());
            assert_eq!(pool.status().retained, 1);
            assert!(traces[1].borrow().retained.is_none());
            let stopped_second = traces[1].borrow().stops;
            traces[0].borrow_mut().stop_fail = false;
            drop(pool);
            assert!(traces[0].borrow().retained.is_none());
            assert_eq!(traces[1].borrow().stops, stopped_second);
        }
        // Issue #340: deterministic clock for the bounded-voice watchdog.
        thread_local! {
            static NOW: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
        }
        fn test_clock() -> Instant {
            NOW.with(|now| now.get().expect("test clock is set before use"))
        }
        fn set_clock(base: Instant, millis: u64) {
            NOW.with(|now| now.set(Some(base + Duration::from_millis(millis))));
        }
        #[test]
        fn voice_reported_playing_past_its_bounded_length_is_reset_once_and_never_restarted() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            pool.clock = test_clock;
            let base = Instant::now();
            set_clock(base, 0);
            let millis = bank.get(1).unwrap().playback_millis();
            assert_eq!(millis, 1);
            pool.play(&bank, &group(b'<', &[1]), 0).unwrap();
            // The fake driver never finishes: it models a stalled device.
            set_clock(base, millis + VOICE_OVERRUN_GRACE_MILLIS - 1);
            assert_eq!(pool.poll().unwrap().active, 1);
            assert_eq!((traces[0].borrow().stops, pool.expired), (0, 0));
            set_clock(base, millis + VOICE_OVERRUN_GRACE_MILLIS);
            assert_eq!(
                pool.poll().unwrap(),
                PoolStatus {
                    active: 0,
                    retained: 0,
                    faulted: false,
                    device_unavailable: false,
                }
            );
            assert_eq!((traces[0].borrow().stops, pool.expired), (1, 1));
            assert!(traces[0].borrow().retained.is_none());
            // Later ticks neither stop it again nor replay the expired cue.
            set_clock(base, 3_600_000);
            for _ in 0..8 {
                pool.poll().unwrap();
            }
            assert_eq!((traces[0].borrow().starts, traces[0].borrow().stops), (1, 1));
            assert_eq!(traces[1].borrow().starts, 0);
            // The healthy pool stays usable and the reused slot has a new deadline.
            assert_eq!(
                pool.play(&bank, &group(b'<', &[2]), 0).unwrap(),
                PlayOutcome::Started {
                    sound_id: 2,
                    slot: 0
                }
            );
            assert_eq!(pool.poll().unwrap().active, 1);
            assert_eq!(pool.expired, 1);
        }
        #[test]
        fn naturally_finished_and_explicitly_stopped_voices_are_not_counted_as_expired() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            pool.clock = test_clock;
            let base = Instant::now();
            set_clock(base, 0);
            pool.play(&bank, &group(b'<', &[1]), 0).unwrap();
            pool.play(&bank, &group(b'<', &[2]), 0).unwrap();
            traces[0].borrow_mut().playing = false;
            set_clock(base, 1);
            assert_eq!(pool.poll().unwrap().active, 1);
            pool.stop_all().unwrap();
            set_clock(base, 3_600_000);
            assert_eq!(pool.poll().unwrap().retained, 0);
            assert_eq!(pool.expired, 0);
            assert_eq!((traces[0].borrow().stops, traces[1].borrow().stops), (0, 1));
            assert!(pool.slots.iter().all(|slot| slot.deadline.is_none()));
        }
        #[test]
        fn failed_reset_of_an_overrunning_voice_latches_the_fault_without_another_start() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            pool.clock = test_clock;
            let base = Instant::now();
            set_clock(base, 0);
            pool.play(&bank, &group(b'<', &[1]), 0).unwrap();
            traces[0].borrow_mut().stop_fail = true;
            set_clock(base, MAX_PLAYBACK_MILLIS + VOICE_OVERRUN_GRACE_MILLIS);
            assert!(matches!(pool.poll(), Err(PoolError::Device(_))));
            assert_eq!(
                pool.status(),
                PoolStatus {
                    active: 1,
                    retained: 1,
                    faulted: true,
                    device_unavailable: false,
                }
            );
            assert_eq!(pool.expired, 0);
            assert!(pool.play(&bank, &group(b'<', &[2]), 0).is_err());
            assert_eq!(traces[1].borrow().starts, 0);
            traces[0].borrow_mut().stop_fail = false;
            assert!(pool.stop_all().is_err());
            assert_eq!(pool.status().retained, 0);
        }
        // ---- Issue #341: device lifecycle ----
        fn legacy(pool: &mut Pool<Fake>, bank: &SoundBank, id: u32) -> Result<PlayOutcome, PoolError> {
            pool.play_prepared(bank.get(id).unwrap(), NativeAudioMode::AllowOverlappingVoices)
        }
        fn starts(traces: &[Rc<RefCell<Trace>>]) -> usize {
            traces.iter().map(|t| t.borrow().starts).sum()
        }
        #[test]
        fn no_output_device_drops_cues_without_fault_and_opens_once_per_retry_interval() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            pool.clock = test_clock;
            let base = Instant::now();
            set_clock(base, 0);
            for trace in &traces {
                trace.borrow_mut().no_device = Some(6);
            }
            assert_eq!(
                pool.play(&bank, &group(b'<', &[1]), 0).unwrap(),
                PlayOutcome::DeviceUnavailable
            );
            let silent = PoolStatus {
                active: 0,
                retained: 0,
                faulted: false,
                device_unavailable: true,
            };
            assert_eq!(pool.status(), silent);
            // A full cue batch inside the interval never reaches the driver.
            for _ in 0..256 {
                assert_eq!(
                    pool.play(&bank, &group(b'<', &[1]), 0).unwrap(),
                    PlayOutcome::DeviceUnavailable
                );
                assert_eq!(
                    legacy(&mut pool, &bank, 2).unwrap(),
                    PlayOutcome::DeviceUnavailable
                );
            }
            assert_eq!(starts(&traces), 1);
            assert_eq!(pool.poll().unwrap(), silent);
            assert_eq!(traces.iter().map(|t| t.borrow().polls).sum::<usize>(), 0);
            assert_eq!(traces.iter().map(|t| t.borrow().stops).sum::<usize>(), 0);
            // After the interval exactly one more open is attempted.
            set_clock(base, DEVICE_RETRY_MILLIS);
            for _ in 0..2 {
                assert_eq!(
                    pool.play(&bank, &group(b'<', &[1]), 0).unwrap(),
                    PlayOutcome::DeviceUnavailable
                );
            }
            assert_eq!(starts(&traces), 2);
            assert_eq!(pool.unavailable_cues, 1 + 512 + 2);
            // A device is connected. Cues inside the running interval are
            // still dropped; the first cue after it plays. Nothing dropped
            // earlier is replayed.
            for trace in &traces {
                trace.borrow_mut().no_device = None;
            }
            set_clock(base, 2 * DEVICE_RETRY_MILLIS - 1);
            assert_eq!(
                pool.play(&bank, &group(b'<', &[1]), 0).unwrap(),
                PlayOutcome::DeviceUnavailable
            );
            assert_eq!(starts(&traces), 2);
            set_clock(base, 2 * DEVICE_RETRY_MILLIS);
            assert_eq!(
                pool.play(&bank, &group(b'<', &[1]), 0).unwrap(),
                PlayOutcome::Started {
                    sound_id: 1,
                    slot: 0
                }
            );
            assert_eq!(
                pool.poll().unwrap(),
                PoolStatus {
                    active: 1,
                    retained: 1,
                    faulted: false,
                    device_unavailable: false,
                }
            );
            assert_eq!(starts(&traces), 3);
            pool.stop_all().unwrap();
        }
        #[test]
        fn device_lost_for_new_cues_leaves_already_playing_voices_alone() {
            let bank = bank();
            let (mut pool, traces) = pool(3);
            assert!(matches!(
                legacy(&mut pool, &bank, 1).unwrap(),
                PlayOutcome::Started { slot: 0, .. }
            ));
            traces[1].borrow_mut().no_device = Some(2);
            assert_eq!(
                legacy(&mut pool, &bank, 2).unwrap(),
                PlayOutcome::DeviceUnavailable
            );
            let status = pool.poll().unwrap();
            assert_eq!((status.active, status.retained), (1, 1));
            assert!(status.device_unavailable && !status.faulted);
            traces[0].borrow_mut().playing = false;
            assert_eq!(pool.poll().unwrap().retained, 0);
            assert_eq!(
                (traces[0].borrow().starts, traces[0].borrow().stops),
                (1, 0)
            );
            assert_eq!(traces[2].borrow().starts, 0);
        }
        #[test]
        fn no_device_code_with_a_buffer_still_owned_by_the_driver_is_a_fault() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            traces[0].borrow_mut().start_fail = true;
            traces[0].borrow_mut().start_error = Some(AudioOutputError::Device(DriverError {
                operation: "waveOutWrite",
                code: 6,
            }));
            assert!(matches!(
                legacy(&mut pool, &bank, 1),
                Err(PoolError::Device(_))
            ));
            let status = pool.status();
            assert!(status.faulted && !status.device_unavailable);
            assert_eq!(traces[0].borrow().stops, 1);
            assert!(legacy(&mut pool, &bank, 2).is_err());
            assert_eq!(starts(&traces), 1);
            assert_eq!(pool.unavailable_cues, 0);
        }
        #[test]
        fn device_fault_recovers_only_after_quiet_time_with_nothing_owned_and_only_a_bounded_number_of_times(
        ) {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            pool.clock = test_clock;
            let base = Instant::now();
            set_clock(base, 0);
            legacy(&mut pool, &bank, 1).unwrap();
            legacy(&mut pool, &bank, 2).unwrap();
            // The device is removed while two voices play: the driver errors
            // on voice 0 and refuses to release it.
            traces[0].borrow_mut().poll_fail = true;
            traces[0].borrow_mut().stop_fail = true;
            assert!(pool.poll().is_err());
            assert_eq!(pool.status().retained, 1);
            assert_eq!(traces[1].borrow().stops, 1);
            // Time alone does not re-enable a pool whose voice is still owned.
            set_clock(base, 3 * FAULT_RECOVERY_MILLIS);
            assert!(pool.poll().is_err());
            assert!(legacy(&mut pool, &bank, 2).is_err());
            // The driver lets go. Quiet time counts from the last failure.
            traces[0].borrow_mut().poll_fail = false;
            traces[0].borrow_mut().stop_fail = false;
            set_clock(base, 3 * FAULT_RECOVERY_MILLIS + 1);
            assert!(pool.poll().is_err());
            assert_eq!(pool.status().retained, 0);
            assert!(pool.stop_all().is_err());
            set_clock(base, 4 * FAULT_RECOVERY_MILLIS - 1);
            assert!(pool.poll().is_err());
            assert!(legacy(&mut pool, &bank, 2).is_err());
            assert_eq!((starts(&traces), pool.recoveries), (2, 0));
            set_clock(base, 4 * FAULT_RECOVERY_MILLIS);
            assert_eq!(
                pool.poll().unwrap(),
                PoolStatus {
                    active: 0,
                    retained: 0,
                    faulted: false,
                    device_unavailable: false,
                }
            );
            // Recovery itself restarts nothing; only a new cue plays.
            assert_eq!((starts(&traces), pool.recoveries), (2, 1));
            assert_eq!(
                legacy(&mut pool, &bank, 1).unwrap(),
                PlayOutcome::Started {
                    sound_id: 1,
                    slot: 0
                }
            );
            pool.stop_all().unwrap();
            // A driver that fails again after every re-enable ends disabled.
            traces[0].borrow_mut().start_fail = true;
            let mut now = 4 * FAULT_RECOVERY_MILLIS;
            let mut recovered = 1;
            loop {
                assert!(legacy(&mut pool, &bank, 1).is_err());
                assert_eq!(pool.status().retained, 0);
                now += FAULT_RECOVERY_MILLIS;
                set_clock(base, now);
                if recovered == MAX_FAULT_RECOVERIES {
                    break;
                }
                pool.poll().unwrap();
                recovered += 1;
                assert_eq!(pool.recoveries, recovered);
            }
            for _ in 0..4 {
                now += 3_600_000;
                set_clock(base, now);
                assert!(pool.poll().is_err());
                assert!(legacy(&mut pool, &bank, 2).is_err());
            }
            assert_eq!(pool.recoveries, MAX_FAULT_RECOVERIES);
            assert_eq!(traces[1].borrow().starts, 1);
        }
        #[test]
        fn sixteen_simultaneous_voices_are_independent_and_a_seventeenth_cue_is_dropped() {
            let bank = bank();
            let (mut pool, traces) = pool(MAX_VOICES);
            for slot in 0..MAX_VOICES {
                let id = 1 + (slot as u32 % 2);
                assert_eq!(
                    legacy(&mut pool, &bank, id).unwrap(),
                    PlayOutcome::Started { sound_id: id, slot }
                );
            }
            assert_eq!(legacy(&mut pool, &bank, 1).unwrap(), PlayOutcome::VoiceLimit);
            assert_eq!(pool.poll().unwrap().active, MAX_VOICES);
            // Two voices finish; the other fourteen are neither stopped nor
            // restarted, and only the two freed slots are reused.
            traces[3].borrow_mut().playing = false;
            traces[9].borrow_mut().playing = false;
            assert_eq!(pool.poll().unwrap().active, MAX_VOICES - 2);
            for slot in [3, 9] {
                assert_eq!(
                    legacy(&mut pool, &bank, 2).unwrap(),
                    PlayOutcome::Started { sound_id: 2, slot }
                );
            }
            assert_eq!(legacy(&mut pool, &bank, 2).unwrap(), PlayOutcome::VoiceLimit);
            for (slot, trace) in traces.iter().enumerate() {
                let expected = if slot == 3 || slot == 9 { 2 } else { 1 };
                assert_eq!((trace.borrow().starts, trace.borrow().stops), (expected, 0));
            }
            pool.stop_all().unwrap();
            assert_eq!(pool.status().retained, 0);
            assert!(traces
                .iter()
                .all(|t| t.borrow().stops == 1 && t.borrow().retained.is_none()));
        }
        #[test]
        fn scene_switches_and_exit_release_every_voice_exactly_once() {
            let bank = bank();
            let (mut pool, traces) = pool(4);
            // Each scene switch stops what is playing and leaves the same
            // fixed voices reusable; idle voices are never touched.
            for _ in 0..50 {
                legacy(&mut pool, &bank, 1).unwrap();
                legacy(&mut pool, &bank, 2).unwrap();
                pool.stop_all().unwrap();
                pool.stop_all().unwrap();
                let status = pool.poll().unwrap();
                assert_eq!((status.active, status.retained), (0, 0));
            }
            for (slot, trace) in traces.iter().enumerate() {
                let expected = if slot < 2 { 50 } else { 0 };
                assert_eq!((trace.borrow().starts, trace.borrow().stops), (expected, expected));
                assert_eq!(trace.borrow().polls, 0);
            }
            // Exit with sound still playing: Drop stops the two voices.
            legacy(&mut pool, &bank, 1).unwrap();
            legacy(&mut pool, &bank, 2).unwrap();
            drop(pool);
            for (slot, trace) in traces.iter().enumerate() {
                assert_eq!(trace.borrow().stops, if slot < 2 { 51 } else { 0 });
                assert!(trace.borrow().retained.is_none() && !trace.borrow().playing);
            }
        }
        #[test]
        fn exit_on_a_failing_device_attempts_one_release_per_voice_and_does_not_panic() {
            let bank = bank();
            let (mut pool, traces) = pool(2);
            legacy(&mut pool, &bank, 1).unwrap();
            legacy(&mut pool, &bank, 2).unwrap();
            traces[0].borrow_mut().stop_fail = true;
            traces[1].borrow_mut().stop_fail = true;
            assert!(pool.stop_all().is_err()); // WM_CLOSE
            assert!(pool.poll().is_err()); // one more timer tick while closing
            drop(pool); // window destroyed
            assert_eq!((traces[0].borrow().stops, traces[1].borrow().stops), (3, 3));
            assert_eq!(starts(&traces), 2);
        }
        #[test]
        fn invalid_capacity_creates_no_backends() {
            for cap in [0, MAX_VOICES + 1, usize::MAX] {
                assert!(matches!(
                    Pool::<Fake>::new(cap, || panic!("must not construct")),
                    Err(PoolError::InvalidCapacity)
                ));
            }
        }
    }
}
#[cfg(windows)]
pub use output::{CombatVoicePool, PoolError};

#[cfg(test)]
mod tests {
    use super::super::combat_animation::classify_native_cues;
    use super::super::sprite_actions::FrameEvent;
    use super::*;
    use anyhow::bail;
    pub(super) fn group(marker: u8, ids: &[u32]) -> NativeAudioGroup {
        let events = ids
            .iter()
            .map(|id| match marker {
                b'<' => FrameEvent::Effect(*id),
                b'[' => FrameEvent::Sound(*id),
                _ => FrameEvent::UninterpretedNumeric { marker, value: *id },
            })
            .collect::<Vec<_>>();
        classify_native_cues(&events).unwrap().audio.unwrap()
    }
    fn wav() -> Vec<u8> {
        [
            b"RIFF".as_slice(),
            &40u32.to_le_bytes(),
            b"WAVEfmt ",
            &16u32.to_le_bytes(),
            &1u16.to_le_bytes(),
            &1u16.to_le_bytes(),
            &22050u32.to_le_bytes(),
            &44100u32.to_le_bytes(),
            &2u16.to_le_bytes(),
            &16u16.to_le_bytes(),
            b"data",
            &4u32.to_le_bytes(),
            &[0, 128, 255, 127],
        ]
        .concat()
    }
    pub(super) fn bank(groups: &[NativeAudioGroup]) -> SoundBank {
        SoundBank::load_with(
            groups,
            BankLimits::default(),
            &AtomicBool::new(false),
            |_, _| Ok(wav()),
        )
        .unwrap()
    }
    #[test]
    fn weighted_alternatives_deduplicate_only_io_and_preserve_pcm_generation() {
        let groups = [
            group(b'<', &[2, 1, 2]),
            group(b'[', &[1]),
            group(b'<', &[0]),
        ];
        let mut reads = Vec::new();
        let bank = SoundBank::load_with(
            &groups,
            BankLimits::default(),
            &AtomicBool::new(false),
            |id, limit| {
                reads.push((id, limit));
                Ok(wav())
            },
        )
        .unwrap();
        assert_eq!(reads.iter().map(|p| p.0).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(groups[0].selected(2), Some(2));
        assert_eq!(bank.ids().collect::<Vec<_>>(), [1, 2]);
        assert_eq!(bank.file_bytes(), 96);
        assert_eq!(bank.pcm_bytes(), 8);
        let sound = bank.get(1).unwrap();
        let cloned = sound.clone();
        assert_eq!(sound.pcm_bytes(), [0, 128, 255, 127]);
        assert_eq!(sound.format().sample_rate, 22050);
        assert_eq!(sound.sha256(), <[u8; 32]>::from(Sha256::digest(wav())));
        assert_eq!(cloned.pcm_bytes().as_ptr(), sound.pcm_bytes().as_ptr());
        drop(bank);
        assert_eq!(cloned.pcm_bytes(), [0, 128, 255, 127]);
    }
    #[test]
    fn bank_budget_is_passed_before_read_and_empty_budget_never_starts_next_file() {
        let groups = [group(b'<', &[1, 2])];
        let mut reads = 0;
        let limits = BankLimits {
            max_bytes: 48,
            ..BankLimits::default()
        };
        assert!(
            SoundBank::load_with(&groups, limits, &AtomicBool::new(false), |_, limit| {
                reads += 1;
                assert_eq!(limit, 48);
                Ok(wav())
            })
            .is_err()
        );
        assert_eq!(reads, 1);
        assert!(SoundBank::load_with(
            &groups[..1],
            BankLimits {
                max_bytes: 47,
                ..limits
            },
            &AtomicBool::new(false),
            |_, limit| {
                assert_eq!(limit, 47);
                Ok(wav())
            }
        )
        .is_err());
    }
    #[test]
    fn malformed_missing_empty_and_unsupported_audio_fail_entire_bank() {
        for bytes in [
            vec![],
            b"RIFFjunk".to_vec(),
            {
                let mut b = wav();
                b[20] = 3;
                b
            },
            {
                let mut b = wav();
                b.pop();
                b
            },
        ] {
            assert!(SoundBank::load_with(
                &[group(b'<', &[1])],
                BankLimits::default(),
                &AtomicBool::new(false),
                |_, _| Ok(bytes.clone())
            )
            .is_err());
        }
        assert!(SoundBank::load_with(
            &[group(b'<', &[1])],
            BankLimits::default(),
            &AtomicBool::new(false),
            |_, _| bail!("missing fixture")
        )
        .is_err());
        assert!(PreparedSound::from_wav(0, wav()).is_err());
        assert!(PreparedSound::from_wav(10_000, wav()).is_err());
        assert!(SoundBank::load_with(
            &[group(b'^', &[1])],
            BankLimits::default(),
            &AtomicBool::new(false),
            |_, _| panic!("caret must not read")
        )
        .is_err());
    }
    #[test]
    fn cancellation_and_limits_fail_before_any_unbounded_or_unneeded_read() {
        let groups = [group(b'<', &[1, 2])];
        let cancel = AtomicBool::new(true);
        assert!(
            SoundBank::load_with(&groups, BankLimits::default(), &cancel, |_, _| panic!(
                "cancelled"
            ))
            .is_err()
        );
        cancel.store(false, Ordering::Release);
        let mut calls = 0;
        assert!(
            SoundBank::load_with(&groups, BankLimits::default(), &cancel, |_, _| {
                calls += 1;
                cancel.store(true, Ordering::Release);
                Ok(wav())
            })
            .is_err()
        );
        assert_eq!(calls, 1);
        for limits in [
            BankLimits {
                max_bytes: 0,
                ..BankLimits::default()
            },
            BankLimits {
                max_bytes: MAX_BANK_BYTES + 1,
                ..BankLimits::default()
            },
            BankLimits {
                max_sounds: 1,
                ..BankLimits::default()
            },
            BankLimits {
                max_groups: 0,
                ..BankLimits::default()
            },
        ] {
            assert!(
                SoundBank::load_with(&groups, limits, &AtomicBool::new(false), |_, _| panic!(
                    "invalid limits"
                ))
                .is_err()
            );
        }
    }
    /// PCM8 mono at an arbitrary rate: one byte per frame.
    fn wav_at(sample_rate: u32, frames: usize) -> Vec<u8> {
        let mut bytes = [
            b"RIFF".as_slice(),
            &0u32.to_le_bytes(),
            b"WAVEfmt ",
            &16u32.to_le_bytes(),
            &1u16.to_le_bytes(),
            &1u16.to_le_bytes(),
            &sample_rate.to_le_bytes(),
            &sample_rate.to_le_bytes(),
            &1u16.to_le_bytes(),
            &8u16.to_le_bytes(),
            b"data",
            &(frames as u32).to_le_bytes(),
        ]
        .concat();
        bytes.resize(bytes.len() + frames + (frames & 1), 128);
        let size = (bytes.len() - 8) as u32;
        bytes[4..8].copy_from_slice(&size.to_le_bytes());
        bytes
    }
    /// Issue #340: no admitted cue can hold a voice for an unbounded time.
    #[test]
    fn playback_length_is_bounded_before_a_sound_can_be_published() {
        let sound = PreparedSound::from_wav(1, wav()).unwrap();
        assert_eq!(sound.playback_millis(), 1);
        // Stereo expansion keeps the frame count, hence the same length.
        assert_eq!(sound.with_spatial(64, 64).unwrap().playback_millis(), 1);
        assert_eq!(sound.with_gain(50).unwrap().playback_millis(), 1);
        // The longest original shared cue (Sound/361.wav, 22.3 s) stays valid.
        let long = PreparedSound::from_wav(361, wav_at(11_025, 11_025 * 23)).unwrap();
        assert_eq!(long.playback_millis(), 23_000);
        // A parser-valid 1 Hz file of 61 bytes would otherwise play for 61 s;
        // 16 MiB of it would play for more than six months.
        let limit = (MAX_PLAYBACK_MILLIS / 1000) as usize;
        assert_eq!(
            PreparedSound::from_wav(1, wav_at(1, limit))
                .unwrap()
                .playback_millis(),
            MAX_PLAYBACK_MILLIS
        );
        assert!(PreparedSound::from_wav(1, wav_at(1, limit + 1)).is_err());
        assert!(PreparedSound::from_wav(1, wav_at(8_000, 8_000 * limit + 1)).is_err());
        // The refusal fails the whole bank instead of publishing a subset.
        assert!(SoundBank::load_with(
            &[group(b'<', &[1])],
            BankLimits::default(),
            &AtomicBool::new(false),
            |_, _| Ok(wav_at(1, limit + 1))
        )
        .is_err());
    }
    #[test]
    fn hit_is_never_interpreted_as_audio_and_zero_only_bank_is_valid_empty() {
        let classified = classify_native_cues(&[FrameEvent::Hit]).unwrap();
        assert!(classified.audio.is_none());
        assert_eq!(classified.non_audio, [FrameEvent::Hit]);
        let bank = SoundBank::load_with(
            &[group(b'<', &[0])],
            BankLimits::default(),
            &AtomicBool::new(false),
            |_, _| panic!("silent"),
        )
        .unwrap();
        assert!(bank.is_empty());
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; qualify exact 14 shared unarmed action WAV banks; no device/output"]
    fn real_shared_unarmed_battle_bank_qualifies_all_weighted_members_without_audio_device() {
        use super::super::combat_animation::{CombatAction, QualifiedCombatAnimation};
        use super::super::resources::{SharedArchive, SharedArchiveKind};
        use super::super::sprite_actions::{ActionLimits, SpriteActionCatalog};
        use super::super::world_scene::BaseAppearance;
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set shared root");
        let mut text = SharedArchive::open(&root, SharedArchiveKind::Text).unwrap();
        assert!(text.entry("list.spr").unwrap().size <= 8 * 1024 * 1024);
        let bytes = text.read_entry("list.spr").unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127"
        );
        let catalog = SpriteActionCatalog::parse_for_inspection(
            &bytes,
            ActionLimits {
                max_bytes: 8 * 1024 * 1024,
                ..Default::default()
            },
        )
        .unwrap();
        let mut groups = Vec::new();
        for class in 0..7 {
            for sex in 0..2 {
                for action in [
                    CombatAction::Attack,
                    CombatAction::Damage,
                    CombatAction::Death,
                ] {
                    let animation = QualifiedCombatAnimation::from_catalog(
                        &catalog,
                        BaseAppearance { class, sex },
                        action,
                    )
                    .unwrap();
                    for frame in &animation.animation().frames {
                        if let Some(group) = classify_native_cues(&frame.events).unwrap().audio {
                            groups.push(group);
                        }
                    }
                }
            }
        }
        let bank = SoundBank::load_shared(
            &root,
            &groups,
            BankLimits::default(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(bank.len(), 63);
        assert_eq!(bank.file_bytes(), 2_095_998);
        assert!(bank.pcm_bytes() > 0 && bank.pcm_bytes() < bank.file_bytes());
        for id in bank.ids() {
            let sound = bank.get(id).unwrap();
            assert_eq!(
                (
                    sound.format().channels,
                    sound.format().sample_rate,
                    sound.format().bits_per_sample
                ),
                (1, 22050, 16)
            );
            assert!(!sound.pcm_bytes().is_empty());
            for gain in [0, 37, 50, 100] {
                let scaled = sound.with_gain(gain).unwrap();
                assert_eq!(scaled.sha256(), sound.sha256());
                for (a, b) in sound
                    .pcm_bytes()
                    .chunks_exact(2)
                    .zip(scaled.pcm_bytes().chunks_exact(2))
                {
                    let original = i32::from(i16::from_le_bytes([a[0], a[1]]));
                    let actual = i16::from_le_bytes([b[0], b[1]]);
                    assert_eq!(i32::from(actual), original * i32::from(gain) / 100);
                }
            }
        }
        // Skill sound resources use the same shared-only preparation and PCM
        // gain path as the base banks. No original LiTo pack or device needed.
        let skills: Vec<_> = [97, 710, 717]
            .into_iter()
            .map(|id| {
                classify_native_cues(&[FrameEvent::Sound(id)])
                    .unwrap()
                    .audio
                    .unwrap()
            })
            .collect();
        let skill_bank = SoundBank::load_shared(
            &root,
            &skills,
            BankLimits::default(),
            &AtomicBool::new(false),
        )
        .unwrap();
        for id in [97, 710, 717] {
            let sound = skill_bank.get(id).unwrap();
            let half = sound.with_gain(50).unwrap();
            assert_eq!(half.sha256(), sound.sha256());
            assert_eq!(sound.format().bits_per_sample, 16);
            for (a, b) in sound
                .pcm_bytes()
                .chunks_exact(2)
                .zip(half.pcm_bytes().chunks_exact(2))
            {
                assert_eq!(
                    i16::from_le_bytes([b[0], b[1]]),
                    i16::from_le_bytes([a[0], a[1]]) / 2
                );
            }
        }
        eprintln!(
            "Qualified shared battle bank: {} weighted frame groups, {} WAVs, {} file bytes; no device opened",
            groups.len(),
            bank.len(),
            bank.file_bytes()
        );
    }
}
