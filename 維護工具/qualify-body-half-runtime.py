"""Qualify the bounded catalogue104flag1 implementation; never full engine acceptance."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import shutil
import zipfile

root = Path(__file__).resolve().parents[1]
stage = root / '驗證紀錄/current/native-draw-flags/body-half'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
pins = read(stage / 'strengthened-build-inputs-before.json')
assert len(pins) == 962
for name, expected in pins.items():
    assert sha(root / name) == expected, name
reference_sha = 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
reference = root / 'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'
source = root.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(reference) == sha(source) == reference_sha
mains = read(stage / 'current-main-retention.json')
assert mains['zip_readback_verified']
profiles = {}
with zipfile.ZipFile(stage / 'current-mains.zip') as z:
    for profile in ['Debug', 'Release']:
        main = mains['profiles'][profile]
        assert sha(Path(main['path'])) == main['sha256']
        assert hashlib.sha256(z.read(f'LiToModern-{profile}.exe')).hexdigest().upper() == main['sha256']
        related = read(stage / f'{profile}-strengthened-qualified.json')
        assert related['passed'] == 16 and related['default_live_ignored'] == 2
        assert related['sources_unchanged'] and related['source_manifest_sha256'] == sha(stage / 'strengthened-build-inputs-before.json')
        assert len(related['images']) == 30 and len(related['visible_attachment_controls']) == 2
        for image in related['images']:
            assert sha(stage / f'{profile}-strengthened-evidence' / image['file']) == image['PNG_sha256']
        assert sha(Path(related['product_zip'])) == related['product_zip_sha256']
        profiles[profile] = {'main': main, 'related_tests': 16, 'default_live_ignored': 2,
                             'pixel_and_owner_cases': 30, 'related_receipt_sha256': sha(stage / f'{profile}-strengthened-qualified.json'),
                             'attachment_controls': related['visible_attachment_controls']}
for name, expected in mains['changed_source_snapshots'].items():
    assert sha(stage / 'source-current' / name) == expected
batch = read(stage / 'runtime-revalidation.json')
assert batch['status'] == 'verified_two_profiles_controller_and_two_GUI_scenarios'
assert len(batch['cases']) == 6
assert {(c['profile'], c['kind'], c['scenario']) for c in batch['cases']} == {
    (p, kind, scenario) for p in profiles for kind, scenario in
    [('controller', 'classic-lab'), ('GUI', 'disabled'), ('GUI', 'classic-lab')]}
cases = []
for case in batch['cases']:
    p = Path(case['path']).resolve()
    assert p.is_relative_to(root / '驗證紀錄/current')
    r = read(p / 'receipt.json')
    assert case['full962_sources_unchanged'] and r['inputs_unchanged'] and r['reference_is_current']
    assert r['jar_sha256'] == reference_sha
    server = r['server']
    assert server['actual_client_sessions'] == 4 and server['memberships_empty']
    assert server['classic_world_members_after_disconnect'] == server['authority_mutations'] == 0
    assert not server['sql_access'] and not server['state_persistent']
    for name, expected in r['source_pins'].items():
        assert sha(root / name) == expected, name
    entry = dict(case, actual_sessions=4, receipt_sha256=sha(p / 'receipt.json'))
    if case['kind'] == 'GUI':
        assert r['status'] == 'verified_world_tree_native_GUI_defined_scenario'
        assert r['complete_build_inputs_verified'] and sha(p / 'compilation-inputs.json') == r['compilation_inputs_sha256']
        compilation = read(p / 'compilation-inputs.json')
        assert compilation['files'] == pins and compilation['profile'] == case['profile']
        assert compilation['main_sha256'] == r['exe_sha256'] == profiles[case['profile']]['main']['sha256']
        comparison = read(p / 'world-tree-gui-comparison.json')
        assert comparison['receipt_sha256'] == sha(p / 'receipt.json')
        assert comparison['exact_C_MOVE'] == 2 and comparison['extra_C_MOVE'] == 0
        assert comparison['scenario'] == case['scenario']
        assert r['private_desktop'] and not r['physical_input_verified']
        for step in r['steps']:
            assert sha(Path(step['png'])) == step['png_sha256']
        entry.update(comparison_sha256=sha(p / 'world-tree-gui-comparison.json'),
                     exact_C_MOVE=2, extra_C_MOVE=0, original_GUI_PNGs=len(r['steps']))
    else:
        assert r['status'] == 'verified_roomless_world_entry' and r['test_exit_code'] == r['server_exit_code'] == 0
        related = read(stage / f"{case['profile']}-strengthened-qualified.json")
        product = next(x for x in related['products'] if x['suite'] == 'native_world_entry')
        assert r['test_binary_sha256'] == product['sha256'] == sha(Path(r['immutable_test_binary']))
        comparison = read(p / 'world-entry-comparison.json')
        assert comparison['receipt_sha256'] == sha(p / 'receipt.json')
        actors = []
        for role in ['a', 'b']:
            report = read(p / f'{role}-world-entry.json')
            assert report['classic_world_compatibility_enabled']
            actor = next(x for x in report['render']['actors'] if x['id'] == 1677722876)
            assert actor['rendered'] and actor['layer_error'] is None
            assert actor['body_presentation']['body_half'] and actor['body_draw_style'] == 'Half'
            assert actor['sprite_member'] == '869-0.spr'
            actors.append({'role': role, 'source_sha256': actor['body_source']['sha256'],
                           'report_sha256': sha(p / f'{role}-world-entry.json')})
        entry.update(test_binary_sha256=product['sha256'], actors=actors,
                     comparison_sha256=sha(p / 'world-entry-comparison.json'))
    cases.append(entry)
sources = stage / 'validation-sources'
sources.mkdir(exist_ok=True)
for name in ['qualify-body-half-runtime.py', 'qualify-body-half-related-build.py',
             'run-body-half-runtime-revalidation.ps1', 'verify-native-world-tree-gui.py',
             'verify-native-world-entry.py', 'verify-native-flag1-static.py']:
    shutil.copy2(root / '維護工具' / name, sources / name)
result = {'status': 'qualified_catalogue104_exact1_body_half_both_profiles_limited_scope',
          'qualified_utc': datetime.now(timezone.utc).isoformat(), 'profiles': profiles,
          'source_files': 962, 'changed_compilation_inputs': 7,
          'source_manifest_sha256': sha(stage / 'strengthened-build-inputs-before.json'),
          'reference_sha256': reference_sha, 'cases': cases, 'actual_sessions': 24,
          'controller_sessions': 8, 'GUI_sessions': 16, 'GUI_exact_C_MOVE': 8, 'GUI_extra_C_MOVE': 0,
          'new_original_pixel_and_owner_cases': 60, 'retained_test_and_main_products': 10,
          'initial20_case_attachment_coverage_gap_retained': 'initial-attachment-coverage-audit.json',
          'initial_weak_fixture_excluded_from_final60_pixel_count': True,
          'prior32F277_ED7F_GUI112_sessions_not_transferred': True,
          'limits': ['Exactly catalogue104=1 body RGB555 half; other catalogue flag combinations remain diagnostic.',
                     'Synthetic visible attachment pairing proves separation; not original869 catalogue attachment presence.',
                     'Scalar fragment comparator shares the SPR decoder; no original runtime framebuffer or nativeRGB565 parity.',
                     'WORLD success requires explicit classic compatibility lab flag; actual deployment/defaultWORLD not qualified.',
                     'Private desktop messages only; actualDPI not independently recorded in this batch; physical input/IME/multiDPI pending.',
                     'Current full Rust/TCP regression, same-endpoint reference boundary, clean-host portability, audioGUI andG11 pending.'],
          'launcher_artifact_written': False, 'native_runtime_framebuffer_parity': False,
          'complete_native_flags': False, 'whole_goal_complete': False}
(stage / 'qualified.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'status': result['status'], 'actual_sessions': 24, 'pixel_owner_cases': 60,
                  'GUI_exact_C_MOVE': 8, 'retained_products': 10}, ensure_ascii=False))
