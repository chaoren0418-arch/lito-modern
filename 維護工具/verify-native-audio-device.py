"""Independent original-WAV and actual-output receipt checks; no sound capture."""
from pathlib import Path
import hashlib
import json
import sys
import wave

case = Path(sys.argv[1]).resolve()
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
r = read(case / 'receipt.json')
assert r['status'] == 'verified_actual_native_audio_device_lifecycle'
assert r['test_exit_code'] == 0 and r['inputs_unchanged']
assert sha(Path(r['immutable_test_binary'])) == r['test_binary_sha256']
for name, digest in r['source_pins'].items():
    assert sha(case / 'source-snapshot' / name) == digest
assert not (case / 'test.stderr.txt').read_text(encoding='utf-8-sig').strip()
a = read(case / 'actual-audio-device.json')
assert a == r['report'] and a['status'] == 'verified_actual_WinMM_default_device_lifecycle'
assert not a['device_routing_or_volume_changed'] and not a['recorded_system_audio']
assert not a['physical_audibility_verified'] and not a['native_GUI_verified']
assert not a['device_unplug_or_no_device_verified'] and not a['full_game_complete']
ids = {d['id'] for d in a['devices']}
assert len(ids) == len(a['devices']) and 0xffffffff in ids
assert all(d['is_mapper'] == (d['id'] == 0xffffffff) and d['name'] for d in a['devices'])
assert [s['sound_id'] for s in a['natural_cases']] == [97, 99, 102]
checks = []
for s in a['natural_cases'] + [a['replacement']]:
    path = Path(f"D:/lineage開發/0-客戶端/Sound/{s['sound_id']}.wav")
    pinned = next(v for p, v in r['readonly_resource_pins'].items() if Path(p) == path)
    assert sha(path) == pinned == s['wav_sha256']
    with wave.open(str(path), 'rb') as w:
        channels, rate, width = w.getnchannels(), w.getframerate(), w.getsampwidth()
        count = w.getnframes()
        pcm = w.readframes(count)
    assert len(pcm) == count * channels * width == s['pcm_bytes']
    assert hashlib.sha256(pcm).hexdigest().upper() == s['pcm_sha256']
    assert (channels, rate, width * 8) == (s['channels'], s['sample_rate'], s['bits_per_sample'])
    duration = (count * 1000 + rate - 1) // rate
    assert duration == s['duration_ms'] and duration // 2 <= s['elapsed_ms'] < duration + 5000
    assert s['playing_samples'] > 0 and s['opened_device_id'] in ids
    assert s['finished_once_then_idle'] and not s['owned_device_after_completion']
    checks.append(dict(sound_id=s['sound_id'], original_PCM_equal=True,
                       driver_completed_ms=s['elapsed_ms'], device_id=s['opened_device_id']))
assert len(a['reset_cases']) == 20
assert [s['cycle'] for s in a['reset_cases']] == list(range(20))
assert all(s['stopped_and_idle'] and s['opened_device_id'] in ids for s in a['reset_cases'])
assert a['simultaneous_voices'] == len(a['simultaneous_opened_devices']) == 4
assert all(id in ids for id in a['simultaneous_opened_devices'])
assert a['drop_while_playing_no_panic']
assert a['handles_after_cycles'] <= a['handles_before_warmed_cycles'] + 4
result = dict(status='verified_original_PCM_actual_WinMM_lifecycle_receipt',
              build_profile=r['build_profile'], test_binary_sha256=r['test_binary_sha256'],
              receipt_sha256=sha(case / 'receipt.json'), report_sha256=sha(case / 'actual-audio-device.json'),
              validator_sha256=sha(Path(__file__)), original_PCM_checks=checks,
              reset_cycles=20, simultaneous_voices=4, physical_audibility_verified=False,
              physical_endpoint_identified=False, native_GUI_verified=False,
              unplug_or_no_device_verified=False, full_product_verified=False)
(case / 'audio-device-comparison.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'status': result['status'], 'checks': checks, 'reset_cycles': 20}))
