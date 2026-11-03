"""Require both normal builds and all current handoff/recovery cases before aggregation."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json

root = Path(__file__).resolve().parents[1]
stage = root / '驗證紀錄/current/native-teleport-wire'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
pins_path = stage / 'green-inputs-before.json'
pins = read(pins_path)
assert len(pins) == 965
for name, expected in pins.items():
    assert sha(root / name) == expected, name
profiles = {}
cases = []
for profile in ['Debug', 'Release']:
    fault_path = stage / f'{profile}-green-qualified.json'
    default_path = stage / f'{profile}-default-qualified.json'
    fault, default = read(fault_path), read(default_path)
    assert fault['tests_passed'] == 6 and fault['fault_scenarios'] == 37
    assert fault['synthetic_TCP_connections'] == 41 and fault['authenticated_formal_sessions'] == 0
    assert default['related_tests_passed'] == 22 and default['features'] == []
    assert fault['source_manifest_sha256'] == default['source_manifest_sha256'] == sha(pins_path)
    assert sha(Path(default['normal_main']['path'])) == default['normal_main']['sha256']
    assert sha(Path(default['archive'])) == default['archive_sha256'] and default['archive_readback_verified']
    profiles[profile] = {'main': default['normal_main'], 'fault_tests': 6, 'fault_scenarios': 37,
                         'synthetic_TCP_connections': 41, 'default_related_Rust': 22,
                         'fault_receipt_sha256': sha(fault_path), 'default_receipt_sha256': sha(default_path)}
    for scenario in ['cross-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-late', 'cancel-expiry']:
        suffix = 'cross' if scenario == 'cross-endpoint' else 'recovery-' + scenario
        qualification_path = stage / f'{profile}-real-{suffix}-qualified.json'
        q = read(qualification_path)
        case = Path(q['case']).resolve()
        assert case.is_relative_to(root / '驗證紀錄/current/native-teleport-gui')
        r = read(case / 'receipt.json')
        compilation = read(case / 'compilation-inputs.json')
        assert q['profile'] == profile and r['scenario'] == scenario
        assert q['main_sha256'] == r['exe_sha256'] == default['normal_main']['sha256']
        assert q['default_build_qualification_sha256'] == sha(default_path)
        assert q['receipt_sha256'] == sha(case / 'receipt.json')
        assert r['inputs_unchanged'] and r['complete_build_inputs_verified'] and r['reference_is_current']
        assert compilation['files'] == pins and compilation['main_sha256'] == q['main_sha256']
        assert q['reference_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
        expected_sessions = 6 if scenario == 'cross-endpoint' else 10 if scenario == 'wrong-service' else 8
        assert q['actual_formal_sessions'] == r['server']['actual_client_sessions'] == expected_sessions
        assert r['server']['memberships_empty'] and r['server']['authority_mutations'] == 0
        assert not r['server']['sql_access'] and not r['server']['state_persistent']
        assert q['exact_C_MOVE'] == (14 if scenario == 'cross-endpoint' else 16) and q['extra_C_MOVE'] == 0
        assert q['original_world_PNGs'] == (4 if scenario == 'cross-endpoint' else 6) and q['original_glyph_matches'] == 4
        if scenario == 'cancel-retry':
            assert q['stale_GUI_notifications'] == 64
            assert q['stale_comparison_sha256'] == sha(case / 'teleport-stale-command-comparison.json')
        cases.append({'profile': profile, 'scenario': scenario, 'case': str(case),
                      'qualification_sha256': sha(qualification_path), 'actual_formal_sessions': expected_sessions,
                      'exact_C_MOVE': q['exact_C_MOVE'], 'original_world_PNGs': q['original_world_PNGs'],
                      'original_glyph_matches': 4, 'stale_GUI_notifications': q.get('stale_GUI_notifications', 0)})
assert len(cases) == len({c['case'] for c in cases}) == 14
assert sum(c['actual_formal_sessions'] for c in cases) == 112
assert sum(c['exact_C_MOVE'] for c in cases) == 220
assert sum(c['original_world_PNGs'] for c in cases) == 80
assert sum(c['original_glyph_matches'] for c in cases) == 56
assert sum(c['stale_GUI_notifications'] for c in cases) == 128
result = {'status': 'qualified_both_profiles_retired_F4_guard_and_current_GUI_recovery_limited',
          'qualified_utc': datetime.now(timezone.utc).isoformat(), 'source_files': 965,
          'source_manifest_sha256': sha(pins_path), 'profiles': profiles, 'cases': cases,
          'formal_GUI_sessions': 112, 'synthetic_TCP_connections': 82, 'synthetic_formal_sessions': 0,
          'exact_GUI_C_MOVE': 220, 'extra_GUI_C_MOVE': 0, 'original_world_PNGs': 80,
          'original_glyph_matches': 56, 'stale_GUI_notifications': 128,
          'current_products_retained': 12, 'red_product_retained_separately': True,
          'red_proof': 'red-reproduction.json', 'failing_test_identity': 'red-green-test-identity.json',
          'normal_main_features': [], 'full_library_regression_verified': False,
          'full_current_TCP_regression_verified': False, 'same_endpoint_current_C02_success_verified': False,
          'physical_input_verified': False, 'whole_goal_complete': False,
          'limits': ['Synthetic scope is seeded from C02; formal GUI cases have no hostile wire injection.',
                     'Full cfg(test) library compilation failed allocation and remains unverified.',
                     'Only private desktop GUI messages, actual observed DPI from glyph receipts; physical input/IME/multiDPI pending.',
                     'Same-endpoint C02 channel boundary, other services/resources/audio, clean-host delivery andG11 remain open.',
                     'OldHUD112 and body-half24-session/60-pixel evidence retain their own prior-main scope.']}
(stage / 'qualified.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({k: result[k] for k in ['status','formal_GUI_sessions','synthetic_TCP_connections','exact_GUI_C_MOVE']}, ensure_ascii=False))
