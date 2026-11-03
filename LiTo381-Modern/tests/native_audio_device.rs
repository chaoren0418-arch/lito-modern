//! Opt-in actual WinMM output, not a fake driver or an audibility assertion.
use lito_modern::modern_lito::{audio::parse_pcm_wav, audio_output::{PcmVoice, VoiceStatus}, combat_audio::PreparedSound, resources::{self, SharedSoundKey}};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::{Path, PathBuf}, thread, time::{Duration, Instant}};
use windows::Win32::{Media::Audio::{waveOutGetDevCapsW, waveOutGetNumDevs, WAVEOUTCAPSW}, System::Threading::{GetCurrentProcess, GetProcessHandleCount}};

fn handles() -> u32 {
    let mut count = 0;
    unsafe { GetProcessHandleCount(GetCurrentProcess(), &mut count).unwrap(); }
    count
}
fn load(id: u32) -> PreparedSound {
    PreparedSound::from_wav(id, resources::read_shared_sound_with_limit(
        Path::new(r"D:\lineage開發\0-客戶端"), SharedSoundKey { sound_id: id }, 1 << 20).unwrap()).unwrap()
}
fn natural(voice: &mut PcmVoice, sound: &PreparedSound) -> Value {
    let format = sound.format();
    let duration_ms = (sound.pcm_bytes().len() as u64 * 1000).div_ceil(u64::from(format.byte_rate));
    assert!((50..=5000).contains(&duration_ms), "live sound must be short and nonempty");
    let start = Instant::now();
    voice.start_prepared(sound).unwrap();
    let device = voice.device_id().unwrap().expect("actual opened WinMM device");
    let mut playing = 0;
    loop {
        match voice.poll().unwrap() {
            VoiceStatus::Playing => playing += 1,
            VoiceStatus::Finished => break,
            VoiceStatus::Idle => panic!("submitted sound silently disappeared"),
        }
        assert!(start.elapsed() < Duration::from_millis(duration_ms + 5000));
        thread::sleep(Duration::from_millis(5));
    }
    assert!(playing > 0, "actual driver must own the pending PCM before finishing");
    assert!(start.elapsed().as_millis() >= u128::from(duration_ms / 2));
    assert_eq!(voice.device_id().unwrap(), None);
    assert_eq!(voice.poll().unwrap(), VoiceStatus::Idle);
    json!({"sound_id":sound.sound_id(), "wav_sha256":format!("{:X}",Sha256::digest(resources::read_shared_sound_with_limit(Path::new(r"D:\lineage開發\0-客戶端"),SharedSoundKey{sound_id:sound.sound_id()},1<<20).unwrap())),
        "pcm_sha256":format!("{:X}",Sha256::digest(sound.pcm_bytes())), "pcm_bytes":sound.pcm_bytes().len(), "channels":format.channels,
        "sample_rate":format.sample_rate,"bits_per_sample":format.bits_per_sample,"duration_ms":duration_ms,
        "elapsed_ms":start.elapsed().as_millis(),"playing_samples":playing,"opened_device_id":device,
        "finished_once_then_idle":true,"owned_device_after_completion":false})
}

#[test]
#[ignore = "opens actual default audio device and emits short original game cues; requires owned output directory"]
fn original_PCM_actual_device_completion_reset_replacement_multivoice_and_drop() {
    let root = PathBuf::from(std::env::var("LITO_AUDIO_TEST_ROOT").expect("explicit owned live audio run"));
    let allowed = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap();
    assert!(root.canonicalize().unwrap().starts_with(allowed));
    let count = unsafe { waveOutGetNumDevs() };
    assert!((1..64).contains(&count), "no actual output device: do not report playback success");
    // WinMM may retain WAVE_MAPPER as the open handle's identity; the mapper
    // also has capabilities. It must not be relabelled as a physical speaker.
    let devices: Vec<Value> = (0..count).chain(std::iter::once(u32::MAX)).map(|id| {
        let mut caps = WAVEOUTCAPSW::default();
        assert_eq!(unsafe { waveOutGetDevCapsW(id as usize, &mut caps, std::mem::size_of::<WAVEOUTCAPSW>() as u32) }, 0);
        let name = caps.szPname;
        json!({"id":id,"is_mapper":id==u32::MAX,"name":String::from_utf16_lossy(&name[..name.iter().position(|x|*x==0).unwrap_or(name.len())])})
    }).collect();
    let sounds = [load(97), load(99), load(102)];
    let mut voice = PcmVoice::new();
    assert_eq!(voice.device_id().unwrap(), None);
    let natural_cases: Vec<Value> = sounds.iter().map(|s| natural(&mut voice, s)).collect();
    for case in &natural_cases { assert!(devices.iter().any(|d|d["id"]==case["opened_device_id"]), "unknown WinMM identity: {case}"); }
    let before = handles();
    let mut reset_cases = Vec::new();
    for cycle in 0..20 {
        voice.start_prepared(&sounds[cycle % sounds.len()]).unwrap();
        let id = voice.device_id().unwrap().unwrap();
        voice.stop().unwrap();
        assert_eq!(voice.device_id().unwrap(), None);
        assert_eq!(voice.poll().unwrap(), VoiceStatus::Idle);
        reset_cases.push(json!({"cycle":cycle,"opened_device_id":id,"stopped_and_idle":true}));
    }
    voice.start_prepared(&sounds[0]).unwrap();
    assert_eq!(voice.poll().unwrap(), VoiceStatus::Playing);
    voice.start_prepared(&sounds[1]).unwrap();
    let replacement = natural(&mut voice, &sounds[2]); // stops the second before queuing third
    let mut voices: Vec<PcmVoice> = (0..4).map(|_|PcmVoice::new()).collect();
    for (i,v) in voices.iter_mut().enumerate() { v.start_prepared(&sounds[i % sounds.len()]).unwrap(); }
    let multi_ids: Vec<u32> = voices.iter().map(|v|v.device_id().unwrap().unwrap()).collect();
    for v in &mut voices { assert_eq!(v.poll().unwrap(), VoiceStatus::Playing); }
    for v in &mut voices { v.stop().unwrap(); assert_eq!(v.device_id().unwrap(),None); }
    drop(voices);
    {
        let mut drop_voice=PcmVoice::new();
        let bytes=resources::read_shared_sound_with_limit(Path::new(r"D:\lineage開發\0-客戶端"),SharedSoundKey{sound_id:102},1<<20).unwrap();
        drop_voice.start(&parse_pcm_wav(&bytes).unwrap()).unwrap();
        drop(bytes); // borrowed-wave API must retain its own copy
        assert_eq!(drop_voice.poll().unwrap(),VoiceStatus::Playing);
    }
    drop(voice);
    thread::sleep(Duration::from_millis(100));
    let after=handles();
    assert!(after <= before + 4, "unexpected handle growth: {before} -> {after}");
    let report=json!({"status":"verified_actual_WinMM_default_device_lifecycle","devices":devices,
        "natural_cases":natural_cases,"reset_cases":reset_cases,"replacement":replacement,
        "simultaneous_voices":4,"simultaneous_opened_devices":multi_ids,"drop_while_playing_no_panic":true,
        "handles_before_warmed_cycles":before,"handles_after_cycles":after,
        "device_routing_or_volume_changed":false,"recorded_system_audio":false,
        "physical_audibility_verified":false,"native_GUI_verified":false,
        "device_unplug_or_no_device_verified":false,"full_game_complete":false});
    std::fs::write(root.join("actual-audio-device.json"),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}
