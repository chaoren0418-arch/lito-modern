"""Bind a real handoff/recovery GUI run to the default-feature wire-fix build."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
root = Path(__file__).resolve().parents[1]
stage = root / '驗證紀錄/current/native-teleport-wire'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
ap = argparse.ArgumentParser()
ap.add_argument('case', type=Path)
ap.add_argument('--profile', choices=['Debug', 'Release'], required=True)
a = ap.parse_args()
case = a.case.resolve(strict=True)
assert case.is_relative_to(root / '驗證紀錄/current/native-teleport-gui')
r = read(case / 'receipt.json')
q = read(stage / f'{a.profile}-default-qualified.json')
pins = read(stage / 'green-inputs-before.json')
assert q['source_inputs_unchanged'] and q['features'] == [] and q['related_tests_passed'] == 22
assert q['source_manifest_sha256'] == sha(stage / 'green-inputs-before.json')
scenario = r['scenario']
assert scenario in ['cross-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-late', 'cancel-expiry']
cross = scenario == 'cross-endpoint'
assert r['status'] == ('verified_native_teleport_GUI_cross_endpoint_shared_battle' if cross
                       else 'verified_native_teleport_GUI_' + scenario + '_and_fresh_reentry')
assert r['inputs_unchanged'] and r['reference_is_current'] and r['complete_build_inputs_verified']
assert r['exe_sha256'] == q['normal_main']['sha256'] == sha(Path(q['normal_main']['path']))
assert sha(case / 'compilation-inputs.json') == r['compilation_inputs_sha256']
compilation = read(case / 'compilation-inputs.json')
assert compilation['files'] == pins and compilation['profile'] == a.profile
assert compilation['main_sha256'] == r['exe_sha256'] and len(pins) == 965
for name, expected in pins.items():
    assert sha(root / name) == expected, name
for name, expected in r['source_pins'].items():
    assert sha(root / name) == expected
receipt_sha = sha(case / 'receipt.json')
authority_path = case / ('teleport-gui-authority-comparison.json' if cross else 'teleport-recovery-gui-comparison.json')
authority_validator = 'verify-native-teleport-gui-authority.py' if cross else 'verify-native-teleport-recovery-gui.py'
authority = read(authority_path)
glyph = read(case / 'teleport-glyph-comparison.json')
scenes = read(case / 'world-scene-pixel-comparison.json')
assert authority['receipt_sha256'] == glyph['receipt_sha256'] == scenes['receipt_sha256'] == receipt_sha
sessions = 6 if cross else 10 if scenario == 'wrong-service' else 8
assert authority['actual_client_sessions'] == sessions and glyph['matched_glyphs'] == 4 and scenes['images'] == (4 if cross else 6)
stale = None
if scenario == 'cancel-retry':
    stale = read(case / 'teleport-stale-command-comparison.json')
    assert r['replay_stale_commands'] and stale['receipt_sha256'] == receipt_sha
    assert stale['status'] == 'verified_64_stale_and_duplicate_GUI_notifications_without_extra_formal_requests'
steps = [s for s in r['steps'] if s['step'] == 'GUI-key-walk']
assert len(steps) == (14 if cross else 16)
headings = {0x68: 0, 0x26: 1, 0x66: 2, 0x27: 3, 0x62: 4, 0x28: 5, 0x64: 6, 0x25: 7}
directions = [(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)]
expected = []
for s in steps:
    heading = headings[s['key']]
    dx, dy = directions[heading]
    assert s['after'] == [s['before'][0] + dx, s['before'][1] + dy]
    expected.append(bytes([0x0a]) + int(s['before'][0]).to_bytes(2, 'little')
                    + int(s['before'][1]).to_bytes(2, 'little') + bytes([heading]))
cap = next((Path(r['runtime']) / 'captures').glob('*/frames.jsonl'))
frames = [json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()]
actual = [bytes.fromhex(x['plain']) for x in frames
          if x['direction'] == 'client_to_server' and x['plain'].startswith('0A ')]
assert actual == expected
out = {'status': 'qualified_default_feature_wire_fix_real_cross_endpoint_GUI' if cross else 'qualified_default_feature_wire_fix_real_recovery_GUI', 'profile': a.profile,
       'scenario': scenario,
       'case': str(case), 'main_sha256': r['exe_sha256'], 'source_files': 965, 'source_inputs_unchanged': True,
       'receipt_sha256': receipt_sha, 'capture_sha256': sha(cap), 'reference_sha256': r['jar_sha256'],
       'default_build_qualification_sha256': sha(stage / f'{a.profile}-default-qualified.json'),
       'actual_formal_sessions': sessions, 'exact_C_MOVE': len(steps), 'extra_C_MOVE': 0,
       'original_world_PNGs': scenes['images'], 'original_glyph_matches': 4,
       'stale_GUI_notifications': 64 if stale else 0,
       'stale_comparison_sha256': sha(case / 'teleport-stale-command-comparison.json') if stale else None,
       'authority_comparison_sha256': sha(authority_path),
       'glyph_comparison_sha256': sha(case / 'teleport-glyph-comparison.json'),
       'scene_comparison_sha256': sha(case / 'world-scene-pixel-comparison.json'),
       'synthetic_faults_injected': False, 'physical_input_verified': False,
       'full_library_regression_verified': False, 'whole_goal_complete': False}
(case / 'wire-fix-default-source-qualification.json').write_text(json.dumps(out, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
suffix = 'cross' if cross else 'recovery-' + scenario
(stage / f'{a.profile}-real-{suffix}-qualified.json').write_text(json.dumps(out, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
dest = case / 'validation-sources'
dest.mkdir(exist_ok=True)
for name in [Path(__file__).name, authority_validator,
             'verify-teleport-glyph-pixels.py', 'verify-current-hud-teleport-scenes.py'] + (['verify-native-teleport-stale-commands.py'] if stale else []):
    p = root / '維護工具' / name
    shutil.copy2(p, dest / (sha(p)[:16] + '-' + p.name))
print(json.dumps({k: out[k] for k in ['status','profile','actual_formal_sessions','exact_C_MOVE']}, ensure_ascii=False))
