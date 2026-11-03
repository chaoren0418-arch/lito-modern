"""Read back the isolated geometry/pixel candidate without accepting a scene."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', type=int, required=True)
ap.add_argument('--terminal-exit-code', type=int, required=True)
a = ap.parse_args()
assert a.terminal_exit_code == 0
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-background-viewport-candidate')
r = read(run / 'receipt.json')
assert r['status'] == 'isolated_background_viewport_candidate_component_passed'
assert r['owned_process'] is None and all(v['exit_code'] == 0 for v in r['processes'])
original = Path(r['original_qualification'])
assert sha(original) == r['original_qualification_sha256']
assert read(original)['source_manifest_sha256'] == r['source_manifest_sha256']
pins = read(run / 'compiled-inputs.json')
assert len(pins) == 1025 and all(sha(ROOT / name) == h for name, h in pins.items())
archived = read(run / 'readback.json')
assert sha(run / 'raw.zip') == archived['archive_sha256']
with zipfile.ZipFile(run / 'raw.zip') as z:
    assert len(z.namelist()) == len(archived['files']) and set(z.namelist()) == set(archived['files'])
    for name, digest in archived['files'].items():
        with z.open(name) as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == digest
candidate = ROOT / '維護工具/candidates/native-background-viewport'
for name, digest in r['candidate_files'].items():
    assert sha(run / ('candidate-' + name)) == sha(candidate / name) == digest
meta = read(candidate / 'metadata.json')
for name, digest in meta['base_hashes'].items():
    assert sha(candidate / ('base-' + name)) == sha(ROOT / 'LiTo381-Modern/src/modern_lito' / name) == digest
geometry = (candidate / 'native_background_geometry.rs').read_text(encoding='utf-8-sig')
assert geometry.startswith((candidate / 'base-native_background_geometry.rs').read_text(encoding='utf-8-sig'))
bank = (candidate / 'native_background.rs').read_text(encoding='utf-8-sig')
start = bank.index('\n    /// Explicit modern full-extent sampling.')
assert bank[:start].rstrip('\n') + bank[bank.rfind('\n}'):] == (candidate / 'base-native_background.rs').read_text(encoding='utf-8-sig')
for source, name, digest in [
    ('native-background-geometry-original.json', 'geometry-fixture.json', 'F4B9F257B740B197CCE25F9BA2369FF5F13A1F16355BBF27E5F8F4E35BE36091'),
    ('native-background-real-original.json', 'frame-fixture.json', '8CDC0BAC8A9E663964A6C81A0E343C871B1E3CCE71DD6827AA06DAF05037C797'),
]:
    assert sha(run / name) == sha(ROOT / 'LiTo381-Modern/tests/fixtures' / source) == digest
report = read(run / 'candidate-results.json')
assert sha(run / 'candidate-results.json') == r['report_sha256']
assert report['native_geometry_cases'] == 2688 and report['native_ordered_calls'] == 17472
assert report['partition_cases'] == 13440 and report['full_word_comparisons'] == 18552576
assert len(report['frames']) == 36 and not report['native_bank_paint_viewport_method_compiled']
assert {tuple(v['dimensions']) for v in report['frames']} == {(744,552),(800,600),(1024,640)}
for frame in report['frames']:
    assert 0 < frame['calls'] <= 16
    w, h = frame['dimensions']
    assert frame['written_pixels'] == (0 if frame['gate'] == 1 else w*h if frame['gate'] == 0 else (w*h+1)//2)
result = dict(status='independently_read_back_isolated_background_viewport_geometry_and_pixel_candidate',
    original_session_id=a.session_id, observed_terminal_exit_code=0, source_files=1024, compiled_input_files=1025,
    source_manifest_sha256=r['source_manifest_sha256'], candidate_files=r['candidate_files'],
    native_geometry_cases=2688, native_ordered_calls=17472, disjoint_full_extent_partition_cases=13440,
    full_word_comparisons=18552576, real_IMG_frames=36, original_native_whole_frames_preserved=12,
    native_744x552_plan_and_bank_paint_unchanged=True, modern_extent_fills_native_522_clip_omissions=True,
    image_source_scope='fixed original BASE Sprite 800.img/803.img after current native index-miss verification',
    archive_sha256=archived['archive_sha256'], report_sha256=r['report_sha256'],
    candidate_geometry_compiled=True, Bank_paint_viewport_method_compiled=False,
    product_source_changed=False, normal_scene_map303_enabled=False, viewport_camera_owner_contract_verified=False,
    original_pixel_oracle_for_modern_extra_extent=False, actual_formal_sessions=0, GUI_verified=False,
    whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
out = run / 'independent-readback.json'
assert not out.exists()
out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
(run / 'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:result[k] for k in ['status','disjoint_full_extent_partition_cases','full_word_comparisons']}, ensure_ascii=False))
