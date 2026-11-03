//! One bounded Windows PCM voice. No filenames, game hooks, callbacks or worker
//! loops: start copies borrowed PCM, start_prepared retains verified shared PCM,
//! and the caller polls on its existing tick.
//! Cancellation/Drop reset, unprepare, then close before releasing the buffer.
//!
//! Device failures are returned, not turned into silent playback success. If a
//! broken driver refuses final cleanup even during Drop, the one retained
//! allocation is intentionally leaked rather than freed while the OS owns it.

use std::cell::UnsafeCell;
use std::fmt;
use std::mem::size_of;

use windows::core::PSTR;
use windows::Win32::Media::Audio::{
    waveOutClose, waveOutGetID, waveOutOpen, waveOutPrepareHeader, waveOutReset, waveOutUnprepareHeader,
    waveOutWrite, CALLBACK_NULL, HWAVEOUT, WAVEFORMATEX, WAVEHDR, WAVERR_STILLPLAYING, WAVE_MAPPER,
};

use super::audio::{PcmFormat, PcmWave, MAX_WAVE_BYTES};
use super::combat_audio::PreparedSound;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoiceStatus {
    Idle,
    Playing,
    /// Reported once, after the driver has released the header and device.
    Finished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriverError {
    pub operation: &'static str,
    pub code: u32,
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} failed (MMRESULT {})", self.operation, self.code)
    }
}
impl std::error::Error for DriverError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AudioOutputError {
    EmptyPcm,
    AllocationFailed,
    InputTooLarge,
    Device(DriverError),
    StartAndCleanup {
        start: DriverError,
        cleanup: DriverError,
    },
}

impl fmt::Display for AudioOutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPcm => write!(f, "cannot play an empty PCM effect"),
            Self::AllocationFailed => write!(f, "unable to allocate bounded PCM playback buffer"),
            Self::InputTooLarge => write!(f, "PCM playback exceeds 16 MiB"),
            Self::Device(error) => error.fmt(f),
            Self::StartAndCleanup { start, cleanup } => {
                write!(f, "{start}; cleanup also failed: {cleanup}")
            }
        }
    }
}
impl std::error::Error for AudioOutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Device(error) => Some(error),
            Self::StartAndCleanup { start, .. } => Some(start),
            _ => None,
        }
    }
}

/// Main-thread-owned, single-voice output. It is not Send/Sync because the
/// driver holds raw pointers into its state; do not move control to a worker.
pub struct PcmVoice {
    inner: Voice<NativeWaveOut>,
}

impl PcmVoice {
    /// Does not open an audio device or emit sound.
    pub fn new() -> Self {
        Self {
            inner: Voice::new(NativeWaveOut),
        }
    }
    /// Replaces a previous effect only after it has been safely stopped.
    /// The input may be dropped as soon as this returns.
    pub fn start(&mut self, wave: &PcmWave<'_>) -> Result<(), AudioOutputError> {
        self.inner.start(wave)
    }
    /// Queue worker-qualified PCM without file IO, parsing or copying its
    /// samples on the UI thread. Retains this individual immutable WAV's Arc
    /// until WinMM releases the header. The old borrowed-wave API still copies.
    pub fn start_prepared(&mut self, sound: &PreparedSound) -> Result<(), AudioOutputError> {
        self.inner.start_prepared(sound)
    }
    /// One non-blocking driver query; never spins or sleeps. The window's
    /// existing tick should call this independently of focus and WM_PAINT.
    pub fn poll(&mut self) -> Result<VoiceStatus, AudioOutputError> {
        self.inner.poll()
    }
    /// Current WinMM device identity for diagnostics. Does not open a device,
    /// change routing/volume, or read audio samples. None means no owned voice.
    /// The result may be WAVE_MAPPER (u32::MAX), not a physical device index.
    /// Other indices are valid only for the current host/device inventory.
    pub fn device_id(&self) -> Result<Option<u32>, AudioOutputError> {
        let Some(playback) = &self.inner.playback else {
            return Ok(None);
        };
        let mut id = 0;
        // SAFETY: playback retains its open handle until stop/poll releases it;
        // this main-thread-owned voice cannot be closed concurrently.
        result("waveOutGetID", unsafe { waveOutGetID(playback.device, &mut id) })
            .map_err(AudioOutputError::Device)?;
        Ok(Some(id))
    }
    /// Explicit shutdown lets the caller report device errors before closing
    /// its UI. Drop additionally retries cleanup as a safety net.
    pub fn stop(&mut self) -> Result<(), AudioOutputError> {
        self.inner.stop_inner().map_err(AudioOutputError::Device)
    }
}

impl Default for PcmVoice {
    fn default() -> Self {
        Self::new()
    }
}

// A private backend boundary allows deterministic lifecycle/error tests without
// opening a real device or emitting sound. Pointer lifetime is owned by Voice.
trait WaveOutApi {
    fn open(&mut self, format: &WAVEFORMATEX) -> Result<HWAVEOUT, DriverError>;
    fn prepare(&mut self, device: HWAVEOUT, header: *mut WAVEHDR) -> u32;
    fn write(&mut self, device: HWAVEOUT, header: *mut WAVEHDR) -> u32;
    fn reset(&mut self, device: HWAVEOUT) -> u32;
    fn unprepare(&mut self, device: HWAVEOUT, header: *mut WAVEHDR) -> u32;
    fn close(&mut self, device: HWAVEOUT) -> u32;
}

struct NativeWaveOut;
impl WaveOutApi for NativeWaveOut {
    fn open(&mut self, format: &WAVEFORMATEX) -> Result<HWAVEOUT, DriverError> {
        let mut handle = HWAVEOUT::default();
        // SAFETY: a complete validated PCM format and writable handle storage.
        // CALLBACK_NULL has no function/window/event pointer to retain. Windows
        // permits freeing WAVEFORMATEX as soon as waveOutOpen returns.
        result("waveOutOpen", unsafe {
            waveOutOpen(
                Some(&mut handle),
                WAVE_MAPPER,
                format,
                None,
                None,
                CALLBACK_NULL,
            )
        })?;
        Ok(handle)
    }
    fn prepare(&mut self, device: HWAVEOUT, header: *mut WAVEHDR) -> u32 {
        // SAFETY: Voice retains the aligned header and PCM until unprepared.
        unsafe { waveOutPrepareHeader(device, header, size_of::<WAVEHDR>() as u32) }
    }
    fn write(&mut self, device: HWAVEOUT, header: *mut WAVEHDR) -> u32 {
        // SAFETY: exactly one successfully prepared, owned header is queued.
        unsafe { waveOutWrite(device, header, size_of::<WAVEHDR>() as u32) }
    }
    fn reset(&mut self, device: HWAVEOUT) -> u32 {
        // SAFETY: device remains open until cleanup has released the header.
        unsafe { waveOutReset(device) }
    }
    fn unprepare(&mut self, device: HWAVEOUT, header: *mut WAVEHDR) -> u32 {
        // SAFETY: the same retained prepared header. STILLPLAYING leaves its
        // allocation alive and is handled without accessing driver-owned flags.
        unsafe { waveOutUnprepareHeader(device, header, size_of::<WAVEHDR>() as u32) }
    }
    fn close(&mut self, device: HWAVEOUT) -> u32 {
        // SAFETY: called only after all queued/prepared buffers are released.
        unsafe { waveOutClose(device) }
    }
}

fn result(operation: &'static str, code: u32) -> Result<(), DriverError> {
    if code == 0 {
        Ok(())
    } else {
        Err(DriverError { operation, code })
    }
}

fn native_format(format: PcmFormat) -> WAVEFORMATEX {
    WAVEFORMATEX {
        wFormatTag: 1,
        nChannels: format.channels,
        nSamplesPerSec: format.sample_rate,
        nAvgBytesPerSec: format.byte_rate,
        nBlockAlign: format.block_align,
        wBitsPerSample: format.bits_per_sample,
        cbSize: 0,
    }
}

// WAVEHDR is packed in windows-rs; give the stable heap address native pointer
// alignment. UnsafeCell permits WinMM to update fields without shared Rust refs.
#[repr(C, align(8))]
struct Header(UnsafeCell<WAVEHDR>);

enum PlaybackPcm {
    Copied(Vec<u8>),
    Shared(PreparedSound),
}
impl PlaybackPcm {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Copied(bytes) => bytes,
            Self::Shared(sound) => sound.pcm_bytes(),
        }
    }
    fn as_ptr(&self) -> *const u8 {
        self.bytes().as_ptr()
    }
}

struct Playback {
    device: HWAVEOUT,
    header: Box<Header>,
    // Never resize or expose mutable access after header.lpData is assigned.
    _pcm: PlaybackPcm,
    prepared: bool,
    submitted: bool,
}

struct Voice<B: WaveOutApi> {
    backend: B,
    playback: Option<Playback>,
}

impl<B: WaveOutApi> Voice<B> {
    fn new(backend: B) -> Self {
        Self {
            backend,
            playback: None,
        }
    }

    fn start(&mut self, wave: &PcmWave<'_>) -> Result<(), AudioOutputError> {
        if wave.pcm_bytes().is_empty() {
            return Err(AudioOutputError::EmptyPcm);
        }
        if wave.pcm_bytes().len() > MAX_WAVE_BYTES {
            return Err(AudioOutputError::InputTooLarge);
        }
        self.stop_inner().map_err(AudioOutputError::Device)?;
        let mut pcm = Vec::new();
        pcm.try_reserve_exact(wave.pcm_bytes().len())
            .map_err(|_| AudioOutputError::AllocationFailed)?;
        pcm.extend_from_slice(wave.pcm_bytes());
        self.begin(wave.format(), PlaybackPcm::Copied(pcm))
    }

    fn start_prepared(&mut self, sound: &PreparedSound) -> Result<(), AudioOutputError> {
        // PreparedSound can only be constructed by the strict PCM parser.
        // Its file and individual PCM extent are already <= 16 MiB.
        self.stop_inner().map_err(AudioOutputError::Device)?;
        self.begin(sound.format(), PlaybackPcm::Shared(sound.clone()))
    }

    fn begin(&mut self, format: PcmFormat, pcm: PlaybackPcm) -> Result<(), AudioOutputError> {
        let header = Box::new(Header(UnsafeCell::new(WAVEHDR {
            // waveOut output data is input-only to the driver; unlike the
            // mutable WAVEHDR, PCM is never written by WinMM. The API's legacy
            // LPSTR type does not imply writable application sample storage.
            lpData: PSTR(pcm.as_ptr().cast_mut()),
            dwBufferLength: pcm.bytes().len() as u32,
            ..WAVEHDR::default()
        })));
        let device = self
            .backend
            .open(&native_format(format))
            .map_err(AudioOutputError::Device)?;
        self.playback = Some(Playback {
            device,
            header,
            _pcm: pcm,
            prepared: false,
            submitted: false,
        });
        let playback = self.playback.as_mut().unwrap();
        let header = playback.header.0.get();
        let prepare = result("waveOutPrepareHeader", self.backend.prepare(device, header));
        if let Err(error) = prepare {
            return Err(self.cleanup_start_error(error));
        }
        playback.prepared = true;
        let write = result("waveOutWrite", self.backend.write(device, header));
        if let Err(error) = write {
            return Err(self.cleanup_start_error(error));
        }
        playback.submitted = true;
        Ok(())
    }

    fn cleanup_start_error(&mut self, start: DriverError) -> AudioOutputError {
        match self.stop_inner() {
            Ok(()) => AudioOutputError::Device(start),
            Err(cleanup) => AudioOutputError::StartAndCleanup { start, cleanup },
        }
    }

    fn poll(&mut self) -> Result<VoiceStatus, AudioOutputError> {
        let Some(playback) = self.playback.as_mut() else {
            return Ok(VoiceStatus::Idle);
        };
        if playback.prepared {
            let code = self
                .backend
                .unprepare(playback.device, playback.header.0.get());
            if code == WAVERR_STILLPLAYING {
                return Ok(VoiceStatus::Playing);
            }
            result("waveOutUnprepareHeader", code).map_err(AudioOutputError::Device)?;
            playback.prepared = false;
            playback.submitted = false;
        }
        result("waveOutClose", self.backend.close(playback.device))
            .map_err(AudioOutputError::Device)?;
        self.playback = None; // Driver no longer owns either allocation.
        Ok(VoiceStatus::Finished)
    }

    fn stop_inner(&mut self) -> Result<(), DriverError> {
        let Some(playback) = self.playback.as_mut() else {
            return Ok(());
        };
        if playback.submitted {
            result("waveOutReset", self.backend.reset(playback.device))?;
            playback.submitted = false;
        }
        if playback.prepared {
            result(
                "waveOutUnprepareHeader",
                self.backend
                    .unprepare(playback.device, playback.header.0.get()),
            )?;
            playback.prepared = false;
        }
        result("waveOutClose", self.backend.close(playback.device))?;
        self.playback = None;
        Ok(())
    }
}

impl<B: WaveOutApi> Drop for Voice<B> {
    fn drop(&mut self) {
        if let Err(error) = self.stop_inner() {
            // A device failing reset/unprepare may still read the memory. Never
            // turn a driver shutdown error into a use-after-free. This is only
            // one <=16 MiB allocation because failed cleanup blocks new start.
            if let Some(playback) = self.playback.take() {
                std::mem::forget(playback);
            }
            // Preserve driver-owned memory before diagnostics. Unlike
            // eprintln!, a closed stderr must not panic during Drop/unwind.
            use std::io::Write;
            let _ = writeln!(
                std::io::stderr().lock(),
                "LiTo audio shutdown: {error}; retaining driver-owned buffer for safety"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::audio::parse_pcm_wav;
    use super::*;
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    #[derive(Default)]
    struct Trace {
        calls: Vec<&'static str>,
        failures: BTreeMap<&'static str, u32>,
        persistent_failures: BTreeMap<&'static str, u32>,
        header_pointers: Vec<(&'static str, usize)>,
        playing: bool,
        pcm: Vec<u8>,
        queued_pointer: usize,
    }
    struct Fake(Rc<RefCell<Trace>>);
    impl Fake {
        fn call(&self, name: &'static str) -> u32 {
            let mut trace = self.0.borrow_mut();
            trace.calls.push(name);
            if let Some(code) = trace.persistent_failures.get(name).copied() {
                code
            } else {
                trace.failures.remove(name).unwrap_or(0)
            }
        }
        fn record_header(&self, operation: &'static str, header: *mut WAVEHDR) {
            self.0
                .borrow_mut()
                .header_pointers
                .push((operation, header as usize));
        }
    }
    impl WaveOutApi for Fake {
        fn open(&mut self, format: &WAVEFORMATEX) -> Result<HWAVEOUT, DriverError> {
            result("waveOutOpen", self.call("open"))?;
            let tag = format.wFormatTag;
            let extra = format.cbSize;
            assert_eq!((tag, extra), (1, 0));
            Ok(HWAVEOUT(1usize as *mut _))
        }
        fn prepare(&mut self, _: HWAVEOUT, header: *mut WAVEHDR) -> u32 {
            self.record_header("prepare", header);
            self.call("prepare")
        }
        fn write(&mut self, _: HWAVEOUT, header: *mut WAVEHDR) -> u32 {
            self.record_header("write", header);
            let code = self.call("write");
            if code == 0 {
                // SAFETY: tests exercise the same retained owned header/data
                // invariant as WinMM, without retaining any borrowed reference.
                let header = unsafe { header.read_unaligned() };
                let pcm = unsafe {
                    std::slice::from_raw_parts(header.lpData.0, header.dwBufferLength as usize)
                };
                let mut trace = self.0.borrow_mut();
                trace.pcm = pcm.to_vec();
                trace.queued_pointer = header.lpData.0 as usize;
                trace.playing = true;
            }
            code
        }
        fn reset(&mut self, _: HWAVEOUT) -> u32 {
            let code = self.call("reset");
            if code == 0 {
                self.0.borrow_mut().playing = false;
            }
            code
        }
        fn unprepare(&mut self, _: HWAVEOUT, header: *mut WAVEHDR) -> u32 {
            self.record_header("unprepare", header);
            let code = self.call("unprepare");
            if code != 0 {
                code
            } else if self.0.borrow().playing {
                WAVERR_STILLPLAYING
            } else {
                0
            }
        }
        fn close(&mut self, _: HWAVEOUT) -> u32 {
            self.call("close")
        }
    }

    fn bytes() -> Vec<u8> {
        // 2 mono PCM16 frames at 22050 Hz.
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
    fn setup() -> (Voice<Fake>, Rc<RefCell<Trace>>) {
        let trace = Rc::new(RefCell::new(Trace::default()));
        (Voice::new(Fake(trace.clone())), trace)
    }

    #[test]
    fn owned_buffer_survives_source_drop_and_poll_releases_completed_voice_once() {
        let (mut voice, trace) = setup();
        assert_eq!(voice.poll().unwrap(), VoiceStatus::Idle);
        {
            let bytes = bytes();
            let wave = parse_pcm_wav(&bytes).unwrap();
            voice.start(&wave).unwrap();
            assert_ne!(
                trace.borrow().queued_pointer,
                wave.pcm_bytes().as_ptr() as usize
            );
        }
        assert_eq!(trace.borrow().pcm, [0, 128, 255, 127]);
        assert_eq!(voice.poll().unwrap(), VoiceStatus::Playing);
        assert!(voice.playback.is_some());
        trace.borrow_mut().playing = false;
        assert_eq!(voice.poll().unwrap(), VoiceStatus::Finished);
        assert_eq!(voice.poll().unwrap(), VoiceStatus::Idle);
        assert_eq!(
            trace.borrow().calls,
            [
                "open",
                "prepare",
                "write",
                "unprepare",
                "unprepare",
                "close"
            ]
        );
    }

    #[test]
    fn prepared_pcm_keeps_same_address_without_copy_through_source_drop_and_stop() {
        let (mut voice, trace) = setup();
        let pointer;
        {
            let sound = PreparedSound::from_wav(1539, bytes()).unwrap();
            pointer = sound.pcm_bytes().as_ptr() as usize;
            voice.start_prepared(&sound).unwrap();
            assert_eq!(trace.borrow().queued_pointer, pointer);
        }
        assert_eq!(
            voice.playback.as_ref().unwrap()._pcm.as_ptr() as usize,
            pointer
        );
        assert_eq!(trace.borrow().pcm, [0, 128, 255, 127]);
        assert_eq!(voice.poll().unwrap(), VoiceStatus::Playing);
        voice.stop_inner().unwrap();
        assert!(voice.playback.is_none());
        assert_eq!(
            trace.borrow().calls,
            [
                "open",
                "prepare",
                "write",
                "unprepare",
                "reset",
                "unprepare",
                "close"
            ]
        );
    }

    #[test]
    fn prepared_failed_cleanup_cannot_replace_retained_arc_until_driver_retires() {
        let (mut voice, trace) = setup();
        let source = PreparedSound::from_wav(1, bytes()).unwrap();
        voice.start_prepared(&source).unwrap();
        let pointer = source.pcm_bytes().as_ptr() as usize;
        trace.borrow_mut().persistent_failures.insert("reset", 5);
        let next = PreparedSound::from_wav(2, bytes()).unwrap();
        assert!(voice.start_prepared(&next).is_err());
        assert_eq!(
            voice.playback.as_ref().unwrap()._pcm.as_ptr() as usize,
            pointer
        );
        assert_eq!(
            trace
                .borrow()
                .calls
                .iter()
                .filter(|call| **call == "open")
                .count(),
            1
        );
        drop(source);
        trace.borrow_mut().persistent_failures.clear();
        voice.stop_inner().unwrap();
        assert!(voice.playback.is_none());
    }

    #[test]
    fn stop_and_replacement_release_in_reset_unprepare_close_order() {
        let (mut voice, trace) = setup();
        let bytes = bytes();
        let wave = parse_pcm_wav(&bytes).unwrap();
        voice.start(&wave).unwrap();
        voice.start(&wave).unwrap();
        voice.stop_inner().unwrap();
        voice.stop_inner().unwrap();
        assert_eq!(
            trace.borrow().calls,
            [
                "open",
                "prepare",
                "write",
                "reset",
                "unprepare",
                "close",
                "open",
                "prepare",
                "write",
                "reset",
                "unprepare",
                "close"
            ]
        );
    }

    #[test]
    fn open_prepare_and_write_errors_are_explicit_and_clean_up() {
        for operation in ["open", "prepare", "write"] {
            let (mut voice, trace) = setup();
            let bytes = bytes();
            let wave = parse_pcm_wav(&bytes).unwrap();
            trace.borrow_mut().failures.insert(operation, 7);
            assert!(matches!(
                voice.start(&wave),
                Err(AudioOutputError::Device(_))
            ));
            assert!(voice.playback.is_none());
            let expected = match operation {
                "open" => vec!["open"],
                "prepare" => vec!["open", "prepare", "close"],
                _ => vec!["open", "prepare", "write", "unprepare", "close"],
            };
            assert_eq!(trace.borrow().calls, expected);
        }
    }

    #[test]
    fn cleanup_failure_retains_buffer_and_prevents_another_voice_until_recovered() {
        for operation in ["reset", "unprepare", "close"] {
            let (mut voice, trace) = setup();
            let bytes = bytes();
            let wave = parse_pcm_wav(&bytes).unwrap();
            voice.start(&wave).unwrap();
            let pointer = trace.borrow().queued_pointer;
            trace.borrow_mut().failures.insert(operation, 5);
            assert!(voice.stop_inner().is_err());
            assert!(voice.playback.is_some());
            trace.borrow_mut().failures.insert(operation, 5);
            assert!(voice.start(&wave).is_err());
            assert_eq!(
                trace
                    .borrow()
                    .calls
                    .iter()
                    .filter(|name| **name == "open")
                    .count(),
                1
            );
            assert_eq!(trace.borrow().queued_pointer, pointer);
            voice.stop_inner().unwrap();
            assert!(voice.playback.is_none());
        }
    }

    #[test]
    fn failed_start_reports_both_original_and_cleanup_failure() {
        let (mut voice, trace) = setup();
        let bytes = bytes();
        let wave = parse_pcm_wav(&bytes).unwrap();
        trace.borrow_mut().failures.insert("write", 7);
        trace.borrow_mut().failures.insert("unprepare", 5);
        assert!(matches!(
            voice.start(&wave),
            Err(AudioOutputError::StartAndCleanup {
                start: DriverError { code: 7, .. },
                cleanup: DriverError { code: 5, .. }
            })
        ));
        assert!(voice.playback.is_some());
        voice.stop_inner().unwrap();
    }

    #[test]
    fn poll_device_errors_do_not_free_still_owned_memory_and_drop_stops() {
        let (mut voice, trace) = setup();
        let bytes = bytes();
        let wave = parse_pcm_wav(&bytes).unwrap();
        voice.start(&wave).unwrap();
        trace.borrow_mut().failures.insert("unprepare", 5);
        assert!(matches!(voice.poll(), Err(AudioOutputError::Device(_))));
        assert!(voice.playback.is_some());
        drop(voice);
        assert_eq!(&trace.borrow().calls[4..], &["reset", "unprepare", "close"]);
    }

    #[test]
    fn persistent_cleanup_failure_is_retried_by_drop_without_advancing_release() {
        for operation in ["reset", "unprepare", "close"] {
            let (mut voice, trace) = setup();
            let source = bytes();
            voice.start(&parse_pcm_wav(&source).unwrap()).unwrap();
            let header_pointer = voice.playback.as_ref().unwrap().header.0.get() as usize;
            let pcm_pointer = voice.playback.as_ref().unwrap()._pcm.as_ptr() as usize;
            trace.borrow_mut().persistent_failures.insert(operation, 5);

            assert!(voice.stop_inner().is_err());
            let retained = voice.playback.as_ref().unwrap();
            assert_eq!(retained.header.0.get() as usize, header_pointer);
            assert_eq!(retained._pcm.as_ptr() as usize, pcm_pointer);
            let calls_before_drop = trace.borrow().calls.len();

            // Exercises the actual take/forget fallback with tiny test PCM.
            // Intentionally retained header/data are NEVER dereferenced after
            // Drop: doing so would turn a cleanup regression into a test UAF.
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                drop(voice);
            }))
            .is_ok());

            let record = trace.borrow();
            assert_eq!(&record.calls[calls_before_drop..], &[operation]);
            assert_eq!(record.persistent_failures.get(operation), Some(&5));
            assert_eq!(record.queued_pointer, pcm_pointer);
            assert!(record
                .header_pointers
                .iter()
                .all(|(_, pointer)| *pointer == header_pointer));
            assert_eq!(record.playing, operation == "reset");
            assert_eq!(
                record.calls.iter().filter(|name| **name == "open").count(),
                1
            );
            match operation {
                "reset" => {
                    assert!(!record.calls.contains(&"unprepare"));
                    assert!(!record.calls.contains(&"close"));
                }
                "unprepare" => assert!(!record.calls.contains(&"close")),
                _ => assert_eq!(
                    record
                        .calls
                        .iter()
                        .filter(|name| **name == "unprepare")
                        .count(),
                    1
                ),
            }
            // Only Playback is retained, not the fake backend/its Rc owner.
            assert_eq!(Rc::strong_count(&trace), 1);
        }
    }

    #[test]
    fn moving_voice_keeps_header_and_pcm_addresses_stable_through_unprepare() {
        let (mut voice, trace) = setup();
        {
            let source = bytes();
            voice.start(&parse_pcm_wav(&source).unwrap()).unwrap();
        }
        let header_pointer = voice.playback.as_ref().unwrap().header.0.get() as usize;
        let pcm_pointer = voice.playback.as_ref().unwrap()._pcm.as_ptr() as usize;

        // Move the controller onto the heap and then back out by value. Neither
        // the separate boxed WAVEHDR nor the Vec allocation may move with it.
        let moved = Box::new(voice);
        assert_eq!(
            moved.playback.as_ref().unwrap().header.0.get() as usize,
            header_pointer
        );
        assert_eq!(
            moved.playback.as_ref().unwrap()._pcm.as_ptr() as usize,
            pcm_pointer
        );
        let mut moved_again = *moved;
        assert_eq!(
            moved_again.playback.as_ref().unwrap().header.0.get() as usize,
            header_pointer
        );
        assert_eq!(
            moved_again.playback.as_ref().unwrap()._pcm.as_ptr() as usize,
            pcm_pointer
        );
        assert_eq!(moved_again.poll().unwrap(), VoiceStatus::Playing);
        moved_again.stop_inner().unwrap();
        assert!(moved_again.playback.is_none());

        let record = trace.borrow();
        assert_eq!(record.queued_pointer, pcm_pointer);
        assert_eq!(
            record.header_pointers,
            [
                ("prepare", header_pointer),
                ("write", header_pointer),
                ("unprepare", header_pointer),
                ("unprepare", header_pointer)
            ]
        );
        assert_eq!(
            record.calls,
            [
                "open",
                "prepare",
                "write",
                "unprepare",
                "reset",
                "unprepare",
                "close"
            ]
        );
    }

    #[test]
    fn poll_close_failure_retries_only_close_and_reports_finished_once() {
        let (mut voice, trace) = setup();
        let source = bytes();
        voice.start(&parse_pcm_wav(&source).unwrap()).unwrap();
        let header_pointer = voice.playback.as_ref().unwrap().header.0.get() as usize;
        let pcm_pointer = voice.playback.as_ref().unwrap()._pcm.as_ptr() as usize;
        trace.borrow_mut().playing = false;
        trace.borrow_mut().failures.insert("close", 5);
        assert_eq!(
            voice.poll(),
            Err(AudioOutputError::Device(DriverError {
                operation: "waveOutClose",
                code: 5,
            }))
        );
        let retained = voice.playback.as_ref().unwrap();
        assert!(!retained.prepared);
        assert!(!retained.submitted);
        assert_eq!(retained.header.0.get() as usize, header_pointer);
        assert_eq!(retained._pcm.as_ptr() as usize, pcm_pointer);

        assert_eq!(voice.poll().unwrap(), VoiceStatus::Finished);
        assert!(voice.playback.is_none());
        assert_eq!(voice.poll().unwrap(), VoiceStatus::Idle);
        drop(voice);
        assert_eq!(
            trace.borrow().calls,
            ["open", "prepare", "write", "unprepare", "close", "close"]
        );
        assert_eq!(
            trace.borrow().header_pointers,
            [
                ("prepare", header_pointer),
                ("write", header_pointer),
                ("unprepare", header_pointer)
            ]
        );
    }
}
