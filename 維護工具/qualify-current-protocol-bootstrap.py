"""Bind current bootstrap tests and an explicit server-harness-only source delta."""
from pathlib import Path
import copy
import hashlib
import json

root = Path(__file__).resolve().parents[1]
stage = root / '驗證紀錄/current/native-teleport-wire'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
save = lambda p, d: Path(p).write_text(json.dumps(d, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
old_dir = stage / 'normal-after-library-test-fixes'
run = stage / 'normal-after-reference-harness-fixes'
old = read(old_dir / 'qualified.json')
r = read(run / 'receipt.json'); pins = read(run / 'source-inputs.json')
assert r['sources_unchanged'] and len(pins) == 968
for name, expected in pins.items():
    assert sha(root / name) == expected, name
before = read(old_dir / 'source-inputs.json')
changed = {name for name in pins if pins[name] != before[name]}
assert changed == {'LiTo381-Modern/tests/oracle/' + name for name in
                   ['NativeLoginHarness.java', 'NativeGameHarness.java', 'NativeRecoveryHarness.java']}
q = copy.deepcopy(old)
q.update(status='qualified_same_client_images_after_test_harness_only_changes',
         source_manifest_sha256=sha(run / 'source-inputs.json'),
         previous_qualification_sha256=sha(old_dir / 'qualified.json'),
         test_only_delta_evidence='驗證紀錄/current/native-protocol-current-bootstrap/qualified.json')
for variant, item in r['profiles'].items():
    assert item['sha256'] == old['profiles'][variant]['sha256'] == sha(item['path'])
    q['profiles'][variant].update(item, entry_log_sha256=sha(run / f'{variant}-entry.log'),
                                compiler_log_sha256=sha(run / f'{variant}-compiler.log'))
save(run / 'qualified.json', q)
prior = root / '驗證紀錄/current/default-native-regression/20261015-103432-269-Debug'
products = read(prior / 'product-retention.json')['products']
assert len(products) == 91
for name, item in products.items():
    assert sha(item['path']) == item['sha256'], name
    dep = Path(item['path']).with_suffix('.d'); assert dep.exists()
    text = dep.read_text(encoding='utf-8', errors='replace')
    assert not any(Path(p).name in text for p in changed), name
fixtures = read(prior / 'pet-fixture-inputs.json')
for name, expected in fixtures['files'].items():
    assert sha(root / name) == expected
output = root / '驗證紀錄/current/native-protocol-current-bootstrap'
proof = {'status': 'same_client_and_default_test_images_after_server_test_harness_only_delta',
         'normal_mains_bit_identical_after_unique_entry_recheck': True,
         'default_test_EXEs_checked': 91, 'default_test_EXEs_bit_identical': True,
         'default_test_Cargo_dependencies_include_changed_Java': False,
         'fixed_runtime_pet_inputs_checked': len(fixtures['files']), 'client_Rust_sources_changed': False,
         'Debug375_results_reexecuted': False,
         'retained_Debug_scope': '375 passed/0 failed/19 ignored; original test images retained, no current TCP/GUI pass transfer.',
         'whole_goal_complete': False}
save(output / 'client-test-invariance.json', proof)
combined = []; total = 0
for name in ['20261015-111323-473-Debug', '20261015-111358-975-Release']:
    path = root / '驗證紀錄/current/native-game' / name / 'receipt.json'
    d = read(path)
    assert d['exe_sha256'] == q['profiles'][d['profile']]['sha256']
    assert d['jar_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
    assert len(d['cases']) == 4 and not d['sql_access']
    for source, expected in d['source_pins'].items():
        assert sha(root / source) == expected
    sessions = []
    for case in d['cases']:
        assert case['verified'] and case['server']['actual_client_sessions'] and not case['server']['sql_access']
        mode = case['mode']; captures = list((Path(d['runtime']) / mode / 'captures').glob('*/frames.jsonl'))
        assert len(captures) == 1
        rows = [json.loads(line) for line in captures[0].read_text(encoding='utf-8-sig').splitlines()]
        count = sum(row['direction'] == 'server_to_client' and row.get('note') == 'S_KEY' for row in rows)
        assert count == (1 if mode == 'unauthorized' else 2)
        total += count
        sessions.append({'mode': mode, 'actual_ClientSessions_from_constructor_and_passive_key_frames': count,
                         'capture_sha256': sha(captures[0])})
    combined.append({'profile': d['profile'], 'case': str(path.parent),
                     'receipt_sha256': sha(path), 'modes': sessions})
assert total == 14
result = {'status': 'verified_current_C02_both_mains_GAME_directory_handoff_notice_empty_and_unauthorized',
          'cases': combined, 'scenarios': 8, 'actual_ClientSessions': 14, 'LOGIN_sessions': 8,
          'GAME_sessions': 6, 'SQL_access': False, 'playable_battle_verified': False,
          'GUI_verified': False, 'full_current_TCP_regression_verified': False, 'whole_goal_complete': False,
          'qualifier_sha256': sha(Path(__file__))}
save(output / 'game-qualified.json', result)
print('Qualified GAME8 scenarios/14 actual sessions; current client and91 default test images unchanged.')
