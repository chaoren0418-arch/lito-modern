//! Bounded, borrowed RIFF/WAVE PCM data, independent of any playback device.
//!
//! The verified shared `Sound/1539.wav` is PCM16 mono at 22050 Hz and includes
//! LIST/JUNQ chunks after its data. Never assume a 44-byte WAV header or infer
//! a codec from a filename. Only integer PCM8/PCM16 mono/stereo is supported;
//! ADPCM, float, extensible WAVE, MIDI, MP3 and container variants fail closed.
//! Shared `Sound/10.wav` omits the final odd data chunk's pad byte; that exact
//! EOF-only variant is accepted and labeled, not used for missing middle pads.
//!
//! Container and format contract: Microsoft RIFF and WAVEFORMATEX documentation:
//! https://learn.microsoft.com/en-us/windows/win32/xaudio2/resource-interchange-file-format--riff-
//! https://learn.microsoft.com/en-us/windows/win32/api/mmeapi/ns-mmeapi-waveformatex

use std::fmt;
use std::ops::Range;
use std::time::Duration;

pub const MAX_WAVE_BYTES: usize = 16 * 1024 * 1024;
const MAX_CHUNKS: usize = 4096;
const MAX_FRAMES: u32 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct WaveLimits {
    pub max_input_bytes: usize,
    pub max_chunks: usize,
    pub max_frames: u32,
}

impl Default for WaveLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: MAX_WAVE_BYTES,
            max_chunks: 256,
            max_frames: MAX_FRAMES,
        }
    }
}

impl WaveLimits {
    fn validate(self) -> Result<Self, WaveError> {
        if self.max_input_bytes == 0
            || self.max_input_bytes > MAX_WAVE_BYTES
            || self.max_chunks == 0
            || self.max_chunks > MAX_CHUNKS
            || self.max_frames == 0
            || self.max_frames > MAX_FRAMES
        {
            return Err(WaveError::InvalidLimits);
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcmSampleFormat {
    Unsigned8,
    Signed16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PcmFormat {
    pub channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub sample_format: PcmSampleFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaveChunk {
    pub id: [u8; 4],
    /// File-relative payload range, excluding chunk header and alignment pad.
    pub data_range: Range<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WavePadding {
    Canonical,
    /// Only the final odd-sized `data` chunk ends at exact RIFF/file EOF.
    /// No PCM byte was truncated or synthesized.
    TerminalUnpaddedData,
}

#[derive(Debug)]
pub struct PcmWave<'a> {
    format: PcmFormat,
    pcm: &'a [u8],
    chunks: Vec<WaveChunk>,
    padding: WavePadding,
}

impl<'a> PcmWave<'a> {
    pub fn format(&self) -> PcmFormat {
        self.format
    }
    pub fn pcm_bytes(&self) -> &'a [u8] {
        self.pcm
    }
    /// All top-level chunks, including opaque metadata. No recursive LIST
    /// interpretation or loop/cue playback policy is implied.
    pub fn chunks(&self) -> &[WaveChunk] {
        &self.chunks
    }
    pub fn padding(&self) -> WavePadding {
        self.padding
    }
    pub fn frame_count(&self) -> u32 {
        (self.pcm.len() / usize::from(self.format.block_align)) as u32
    }
    pub fn duration(&self) -> Duration {
        let frames = u64::from(self.frame_count());
        let rate = u64::from(self.format.sample_rate);
        Duration::new(
            frames / rate,
            ((frames % rate) * 1_000_000_000 / rate) as u32,
        )
    }
    /// On-demand signed PCM16 sample; PCM8 is centered at 128 then shifted by
    /// eight bits. No whole-file resampling/allocation is needed to inspect it.
    pub fn sample_i16(&self, frame: u32, channel: u16) -> Option<i16> {
        if frame >= self.frame_count() || channel >= self.format.channels {
            return None;
        }
        let index = frame as usize * usize::from(self.format.block_align)
            + usize::from(channel) * usize::from(self.format.bits_per_sample / 8);
        match self.format.sample_format {
            PcmSampleFormat::Unsigned8 => Some((i16::from(*self.pcm.get(index)?) - 128) << 8),
            PcmSampleFormat::Signed16 => Some(i16::from_le_bytes([
                *self.pcm.get(index)?,
                *self.pcm.get(index + 1)?,
            ])),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum WaveError {
    InvalidLimits,
    LimitExceeded(&'static str),
    Truncated { offset: usize, needed: usize },
    UnsupportedContainer([u8; 4]),
    UnsupportedEncoding(u16),
    UnsupportedPcm { channels: u16, bits: u16 },
    MissingChunk([u8; 4]),
    DuplicateChunk([u8; 4]),
    InvalidData(&'static str),
    AllocationFailed,
}

impl fmt::Display for WaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits => write!(f, "invalid bounded WAVE limits"),
            Self::LimitExceeded(what) => write!(f, "WAVE exceeds {what} limit"),
            Self::Truncated { offset, needed } => {
                write!(f, "truncated WAVE at {offset}, need {needed} bytes")
            }
            Self::UnsupportedContainer(tag) => write!(f, "unsupported audio container {tag:?}"),
            Self::UnsupportedEncoding(tag) => write!(f, "unsupported WAVE encoding {tag:#06x}"),
            Self::UnsupportedPcm { channels, bits } => write!(
                f,
                "unsupported PCM layout: {channels} channels, {bits} bits"
            ),
            Self::MissingChunk(tag) => write!(f, "missing WAVE chunk {tag:?}"),
            Self::DuplicateChunk(tag) => write!(f, "ambiguous duplicate WAVE chunk {tag:?}"),
            Self::InvalidData(what) => write!(f, "invalid WAVE: {what}"),
            Self::AllocationFailed => write!(f, "unable to allocate bounded WAVE metadata"),
        }
    }
}
impl std::error::Error for WaveError {}

fn bytes_at(bytes: &[u8], offset: usize, length: usize) -> Result<&[u8], WaveError> {
    let end = offset
        .checked_add(length)
        .ok_or(WaveError::InvalidData("range overflow"))?;
    bytes.get(offset..end).ok_or(WaveError::Truncated {
        offset,
        needed: length,
    })
}
fn word(bytes: &[u8], offset: usize) -> Result<u16, WaveError> {
    let b = bytes_at(bytes, offset, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn dword(bytes: &[u8], offset: usize) -> Result<u32, WaveError> {
    let b = bytes_at(bytes, offset, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn parse_format(bytes: &[u8]) -> Result<PcmFormat, WaveError> {
    bytes_at(bytes, 0, 16)?;
    let encoding = word(bytes, 0)?;
    if encoding != 1 {
        return Err(WaveError::UnsupportedEncoding(encoding));
    }
    // PCM's canonical 16-byte fmt or WAVEFORMATEX with empty extension.
    if bytes.len() != 16 && !(bytes.len() == 18 && word(bytes, 16)? == 0) {
        return Err(WaveError::InvalidData(
            "PCM fmt must be 16 bytes or 18 bytes with cbSize zero",
        ));
    }
    let channels = word(bytes, 2)?;
    let sample_rate = dword(bytes, 4)?;
    let byte_rate = dword(bytes, 8)?;
    let block_align = word(bytes, 12)?;
    let bits_per_sample = word(bytes, 14)?;
    if !(channels == 1 || channels == 2) || !(bits_per_sample == 8 || bits_per_sample == 16) {
        return Err(WaveError::UnsupportedPcm {
            channels,
            bits: bits_per_sample,
        });
    }
    if !(1..=192_000).contains(&sample_rate) {
        return Err(WaveError::InvalidData("sample rate outside 1..192000 Hz"));
    }
    if block_align != channels * (bits_per_sample / 8) {
        return Err(WaveError::InvalidData(
            "PCM block alignment does not match channels and bits",
        ));
    }
    if byte_rate != sample_rate * u32::from(block_align) {
        return Err(WaveError::InvalidData(
            "PCM byte rate does not match sample rate and block alignment",
        ));
    }
    Ok(PcmFormat {
        channels,
        sample_rate,
        byte_rate,
        block_align,
        bits_per_sample,
        sample_format: if bits_per_sample == 8 {
            PcmSampleFormat::Unsigned8
        } else {
            PcmSampleFormat::Signed16
        },
    })
}

pub fn parse_pcm_wav(bytes: &[u8]) -> Result<PcmWave<'_>, WaveError> {
    parse_pcm_wav_with_limits(bytes, WaveLimits::default())
}

pub fn parse_pcm_wav_with_limits(
    bytes: &[u8],
    limits: WaveLimits,
) -> Result<PcmWave<'_>, WaveError> {
    let limits = limits.validate()?;
    if bytes.len() > limits.max_input_bytes {
        return Err(WaveError::LimitExceeded("input bytes"));
    }
    let header = bytes_at(bytes, 0, 12)?;
    if &header[..4] != b"RIFF" {
        return Err(WaveError::UnsupportedContainer(
            header[..4].try_into().unwrap(),
        ));
    }
    if &header[8..12] != b"WAVE" {
        return Err(WaveError::UnsupportedContainer(
            header[8..12].try_into().unwrap(),
        ));
    }
    // u64 prevents malicious u32 sizes from wrapping on the x86 launcher.
    if u64::from(dword(bytes, 4)?) + 8 != bytes.len() as u64 {
        return Err(WaveError::InvalidData(
            "RIFF size must match the complete file",
        ));
    }
    let mut position = 12usize;
    let mut format = None;
    let mut data = None;
    let mut chunks = Vec::new();
    let mut padding = WavePadding::Canonical;
    while position < bytes.len() {
        if chunks.len() == limits.max_chunks {
            return Err(WaveError::LimitExceeded("chunk count"));
        }
        let chunk_header = bytes_at(bytes, position, 8)?;
        let id: [u8; 4] = chunk_header[..4].try_into().unwrap();
        let size = dword(chunk_header, 4)? as usize;
        let start = position + 8;
        let payload = bytes_at(bytes, start, size)?;
        let end = start + size;
        let next = if size & 1 != 0 && end == bytes.len() && id == *b"data" {
            padding = WavePadding::TerminalUnpaddedData;
            end
        } else {
            bytes_at(bytes, end, size & 1)?; // All other chunks stay WORD-aligned.
            end + (size & 1)
        };
        match &id {
            b"fmt " => {
                if format.is_some() {
                    return Err(WaveError::DuplicateChunk(id));
                }
                format = Some(parse_format(payload)?);
            }
            b"data" => {
                if data.is_some() {
                    return Err(WaveError::DuplicateChunk(id));
                }
                if format.is_none() {
                    return Err(WaveError::InvalidData("data precedes fmt"));
                }
                data = Some(payload);
            }
            _ => {} // Metadata is preserved by range, never fed to PCM playback.
        }
        chunks
            .try_reserve(1)
            .map_err(|_| WaveError::AllocationFailed)?;
        chunks.push(WaveChunk {
            id,
            data_range: start..end,
        });
        position = next;
    }
    let format = format.ok_or(WaveError::MissingChunk(*b"fmt "))?;
    let pcm = data.ok_or(WaveError::MissingChunk(*b"data"))?;
    if pcm.len() % usize::from(format.block_align) != 0 {
        return Err(WaveError::InvalidData("PCM data ends in a partial frame"));
    }
    if pcm.len() / usize::from(format.block_align) > limits.max_frames as usize {
        return Err(WaveError::LimitExceeded("PCM frame count"));
    }
    Ok(PcmWave {
        format,
        pcm,
        chunks,
        padding,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(channels: u16, bits: u16) -> Vec<u8> {
        let align = channels * (bits / 8);
        let mut bytes = 1u16.to_le_bytes().to_vec();
        bytes.extend(channels.to_le_bytes());
        bytes.extend(22050u32.to_le_bytes());
        bytes.extend((22050 * u32::from(align)).to_le_bytes());
        bytes.extend(align.to_le_bytes());
        bytes.extend(bits.to_le_bytes());
        bytes
    }
    fn wave(chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
        let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
        for (tag, payload) in chunks {
            bytes.extend(*tag);
            bytes.extend((payload.len() as u32).to_le_bytes());
            bytes.extend(*payload);
            if payload.len() & 1 != 0 {
                bytes.push(0);
            }
        }
        let size = (bytes.len() - 8) as u32;
        bytes[4..8].copy_from_slice(&size.to_le_bytes());
        bytes
    }

    #[test]
    fn pcm16_stereo_preserves_samples_stride_and_duration() {
        let samples = [-32768i16, 32767, -275, 525];
        let pcm: Vec<_> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        let bytes = wave(&[(b"fmt ", &format(2, 16)), (b"data", &pcm)]);
        let decoded = parse_pcm_wav(&bytes).unwrap();
        assert_eq!(decoded.frame_count(), 2);
        assert_eq!(decoded.format().block_align, 4);
        assert_eq!(decoded.duration(), Duration::from_nanos(90_702));
        assert_eq!(decoded.pcm_bytes(), pcm);
        assert_eq!(decoded.sample_i16(0, 0), Some(-32768));
        assert_eq!(decoded.sample_i16(0, 1), Some(32767));
        assert_eq!(decoded.sample_i16(1, 0), Some(-275));
        assert_eq!(decoded.sample_i16(1, 1), Some(525));
        assert_eq!(decoded.sample_i16(2, 0), None);
        assert_eq!(decoded.sample_i16(0, 2), None);
        assert_eq!(decoded.sample_i16(u32::MAX, u16::MAX), None);
    }

    #[test]
    fn pcm8_padding_and_metadata_do_not_become_audio_samples() {
        let bytes = wave(&[
            (b"JUNK", &[99]),
            (b"fmt ", &format(1, 8)),
            (b"data", &[0, 128, 255]),
            (b"LIST", b"INFO"),
        ]);
        let decoded = parse_pcm_wav(&bytes).unwrap();
        assert_eq!(decoded.frame_count(), 3);
        assert_eq!(decoded.chunks().len(), 4);
        assert_eq!(decoded.chunks()[2].data_range, 54..57);
        assert_eq!(
            (0..3)
                .map(|f| decoded.sample_i16(f, 0).unwrap())
                .collect::<Vec<_>>(),
            [-32768, 0, 32512]
        );
    }

    #[test]
    fn fmt_extension_zero_and_empty_pcm_are_explicitly_supported() {
        let mut fmt = format(1, 16);
        fmt.extend([0, 0]);
        let bytes = wave(&[(b"fmt ", &fmt), (b"data", &[])]);
        let decoded = parse_pcm_wav(&bytes).unwrap();
        assert_eq!(decoded.frame_count(), 0);
        assert_eq!(decoded.duration(), Duration::ZERO);
        assert_eq!(decoded.sample_i16(0, 0), None);
        fmt[16] = 1;
        assert!(parse_pcm_wav(&wave(&[(b"fmt ", &fmt), (b"data", &[])])).is_err());
    }

    #[test]
    fn malformed_truncated_and_overflowing_containers_are_rejected() {
        let valid = wave(&[(b"fmt ", &format(1, 16)), (b"data", &[1, 2])]);
        for length in 0..valid.len() {
            assert!(parse_pcm_wav(&valid[..length]).is_err(), "length {length}");
        }
        let mut bad = valid.clone();
        bad.extend([0]);
        assert!(parse_pcm_wav(&bad).is_err());
        bad = valid.clone();
        bad[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse_pcm_wav(&bad).is_err());
        bad = valid.clone();
        bad[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse_pcm_wav(&bad).is_err());
        for magic in [b"RIFX", b"RF64", b"OggS"] {
            bad = valid.clone();
            bad[..4].copy_from_slice(magic);
            assert!(matches!(
                parse_pcm_wav(&bad),
                Err(WaveError::UnsupportedContainer(_))
            ));
        }
        let mut missing_pad = wave(&[(b"fmt ", &format(1, 8)), (b"data", &[1])]);
        missing_pad.pop();
        let size = (missing_pad.len() - 8) as u32;
        missing_pad[4..8].copy_from_slice(&size.to_le_bytes());
        let decoded = parse_pcm_wav(&missing_pad).unwrap();
        assert_eq!(decoded.padding(), WavePadding::TerminalUnpaddedData);
        assert_eq!(decoded.pcm_bytes(), &[1]);
        let mut missing_metadata_pad = wave(&[
            (b"fmt ", &format(1, 8)),
            (b"data", &[1, 2]),
            (b"JUNK", &[3]),
        ]);
        missing_metadata_pad.pop();
        let size = (missing_metadata_pad.len() - 8) as u32;
        missing_metadata_pad[4..8].copy_from_slice(&size.to_le_bytes());
        assert!(matches!(
            parse_pcm_wav(&missing_metadata_pad),
            Err(WaveError::Truncated { .. })
        ));
    }

    #[test]
    fn eof_padding_exception_never_hides_middle_padding_or_partial_pcm_frames() {
        let mut middle = wave(&[
            (b"fmt ", &format(1, 8)),
            (b"data", &[1]),
            (b"LIST", b"INFO"),
        ]);
        middle.remove(45); // Remove data's required pad before LIST.
        let size = (middle.len() - 8) as u32;
        middle[4..8].copy_from_slice(&size.to_le_bytes());
        assert!(parse_pcm_wav(&middle).is_err());
        let mut partial = wave(&[(b"fmt ", &format(1, 16)), (b"data", &[1])]);
        partial.pop();
        let size = (partial.len() - 8) as u32;
        partial[4..8].copy_from_slice(&size.to_le_bytes());
        assert!(matches!(
            parse_pcm_wav(&partial),
            Err(WaveError::InvalidData("PCM data ends in a partial frame"))
        ));
    }

    #[test]
    fn duplicate_missing_and_out_of_order_required_chunks_fail_closed() {
        let fmt = format(1, 16);
        for chunks in [
            vec![
                (b"fmt ", fmt.as_slice()),
                (b"fmt ", fmt.as_slice()),
                (b"data", &[]),
            ],
            vec![(b"fmt ", fmt.as_slice()), (b"data", &[]), (b"data", &[])],
        ] {
            assert!(matches!(
                parse_pcm_wav(&wave(&chunks)),
                Err(WaveError::DuplicateChunk(_))
            ));
        }
        assert!(matches!(
            parse_pcm_wav(&wave(&[])),
            Err(WaveError::MissingChunk(_))
        ));
        assert!(matches!(
            parse_pcm_wav(&wave(&[(b"fmt ", &fmt)])),
            Err(WaveError::MissingChunk(_))
        ));
        assert!(parse_pcm_wav(&wave(&[(b"data", &[]), (b"fmt ", &fmt)])).is_err());
    }

    #[test]
    fn unsupported_codecs_layouts_and_inconsistent_pcm_fields_are_rejected() {
        for tag in [2u16, 3, 0x11, 0xfffe] {
            let mut fmt = format(1, 16);
            fmt[..2].copy_from_slice(&tag.to_le_bytes());
            assert!(
                matches!(parse_pcm_wav(&wave(&[(b"fmt ", &fmt), (b"data", &[])])), Err(WaveError::UnsupportedEncoding(value)) if value == tag)
            );
        }
        for (channels, bits) in [(0, 16), (3, 16), (1, 24), (1, 32)] {
            assert!(matches!(
                parse_pcm_wav(&wave(&[(b"fmt ", &format(channels, bits)), (b"data", &[])])),
                Err(WaveError::UnsupportedPcm { .. })
            ));
        }
        for offset in [4, 8, 12] {
            let mut fmt = format(1, 16);
            fmt[offset] ^= 0x80;
            assert!(parse_pcm_wav(&wave(&[(b"fmt ", &fmt), (b"data", &[])])).is_err());
        }
        assert!(parse_pcm_wav(&wave(&[(b"fmt ", &format(1, 16)), (b"data", &[1])])).is_err());
    }

    #[test]
    fn all_allocation_and_work_limits_are_enforced() {
        let bytes = wave(&[(b"fmt ", &format(1, 8)), (b"data", &[1, 2, 3])]);
        for limits in [
            WaveLimits {
                max_input_bytes: bytes.len() - 1,
                ..WaveLimits::default()
            },
            WaveLimits {
                max_chunks: 1,
                ..WaveLimits::default()
            },
            WaveLimits {
                max_frames: 2,
                ..WaveLimits::default()
            },
        ] {
            assert!(matches!(
                parse_pcm_wav_with_limits(&bytes, limits),
                Err(WaveError::LimitExceeded(_))
            ));
        }
        assert!(matches!(
            parse_pcm_wav_with_limits(
                &bytes,
                WaveLimits {
                    max_chunks: usize::MAX,
                    ..WaveLimits::default()
                }
            ),
            Err(WaveError::InvalidLimits)
        ));
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; reads only shared Sound/1539.wav"]
    fn real_shared_sound_1539_matches_pcm_reference() {
        use crate::modern_lito::resources::{read_shared_sound, SharedSoundKey};
        use sha2::{Digest, Sha256};
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set shared resource root");
        let bytes = read_shared_sound(root, SharedSoundKey { sound_id: 1539 }).unwrap();
        assert_eq!(bytes.len(), 20134);
        assert_eq!(
            format!("{:X}", Sha256::digest(&bytes)),
            "4BDB0AF502BFF04B83598E1C865365031B960360FE3160CF8A8849B29DE8ED4B"
        );
        let decoded = parse_pcm_wav(&bytes).unwrap();
        assert_eq!(
            decoded.format(),
            PcmFormat {
                channels: 1,
                sample_rate: 22050,
                byte_rate: 44100,
                block_align: 2,
                bits_per_sample: 16,
                sample_format: PcmSampleFormat::Signed16
            }
        );
        assert_eq!(decoded.frame_count(), 9456);
        assert_eq!(decoded.pcm_bytes().len(), 18912);
        assert_eq!(
            format!("{:X}", Sha256::digest(decoded.pcm_bytes())),
            "0D1A69E5506D8B630D577F94CB3E9A7F394A61D415886AA2286186ADA61AA11F"
        );
        assert_eq!(
            (0..16)
                .map(|frame| decoded.sample_i16(frame, 0).unwrap())
                .collect::<Vec<_>>(),
            [-275, -151, 227, 201, 222, -193, -399, -186, 451, 525, 68, -299, -218, 109, 31, -44]
        );
        assert_eq!(
            decoded
                .chunks()
                .iter()
                .map(|chunk| (chunk.id, chunk.data_range.len()))
                .collect::<Vec<_>>(),
            vec![
                (*b"fmt ", 16),
                (*b"data", 18912),
                (*b"LIST", 74),
                (*b"JUNQ", 1088)
            ]
        );
        assert_eq!(decoded.padding(), WavePadding::Canonical);
        eprintln!(
            "Shared Sound/1539.wav: {:?}, {} frames, {:?}; PCM SHA matches independent .NET reference",
            decoded.format(),
            decoded.frame_count(),
            decoded.duration()
        );
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; reads only shared Sound/10.wav"]
    fn real_shared_sound_10_has_explicit_terminal_unpadded_pcm8_variant() {
        use crate::modern_lito::resources::{read_shared_sound, SharedSoundKey};
        use sha2::{Digest, Sha256};
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set shared resource root");
        let bytes = read_shared_sound(root, SharedSoundKey { sound_id: 10 }).unwrap();
        assert_eq!(bytes.len(), 12383);
        assert_eq!(
            format!("{:X}", Sha256::digest(&bytes)),
            "A2BB508ED17087FF8A01C4B04B9051ADE9E3677F20B5CDAA308843467C434B1F"
        );
        let decoded = parse_pcm_wav(&bytes).unwrap();
        assert_eq!(decoded.padding(), WavePadding::TerminalUnpaddedData);
        assert_eq!(
            decoded.format(),
            PcmFormat {
                channels: 1,
                sample_rate: 11025,
                byte_rate: 11025,
                block_align: 1,
                bits_per_sample: 8,
                sample_format: PcmSampleFormat::Unsigned8
            }
        );
        assert_eq!(decoded.frame_count(), 12339);
        assert_eq!(
            format!("{:X}", Sha256::digest(decoded.pcm_bytes())),
            "F9A85FE38F7F3FE52437BDC5740F9A7A20D4413CDFCB7085BAB3688D5184520A"
        );
        assert_eq!(
            &decoded.pcm_bytes()[..16],
            &[129, 127, 128, 128, 128, 128, 128, 128, 127, 129, 128, 127, 128, 128, 128, 128]
        );
        eprintln!(
            "Shared Sound/10.wav: {:?}, {} frames, {:?}",
            decoded.format(),
            decoded.frame_count(),
            decoded.padding()
        );
    }
}
