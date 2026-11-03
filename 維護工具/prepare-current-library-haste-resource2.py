"""Declare two more distinct ignored resource bodies after reading all helpers."""
from pathlib import Path
from datetime import datetime, timezone
import ast
import hashlib
import json
import re
R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
save = lambda path, value: Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
w = read(R / '工作佇列.json')
source = w['current_product_identity']['source_manifest_sha256']
assert source == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
normal = Path(w['current_native_installed_resource_profile_normal_qualification']['run'])
inputs = read(normal / 'compiled-inputs.json')
assert len(inputs) == 1049 and all(sha(R / name) == digest for name, digest in inputs.items())
remaining_path = V / 'native-library-remaining-optin-review/20261020-041932-322366/review.json'
remaining = read(remaining_path)
names = ['modern_lito::haste_effect::tests::real_shared_haste43_effect_frames_and_terminal',
         'modern_lito::greater_haste_effect::tests::real_shared_greater_haste_effect_frames_and_terminal']
selected = [row for row in remaining['tests'] if row['test'] in names]
assert len(selected) == len(set(names)) == 2
motion = w['current_library_motion_resource2']
assert motion['qualified'] and motion['source_manifest_sha256'] == source
assert sha(motion['qualification']) == motion['qualification_sha256']
assert not set(names) & set(motion['names'])
parser_path = R / '維護工具/prepare-current-shared-resource-library-plan.py'
parsed = ast.parse(parser_path.read_text('utf-8-sig'))
function = next(node for node in parsed.body if isinstance(node, ast.FunctionDef) and node.name == 'function_body')
namespace = {'re': re}
exec(compile(ast.Module(body=[function], type_ignores=[]), str(parser_path), 'exec'), namespace)
extract = namespace['function_body']
bodies = {}
for row in selected:
    path = R / row['source']
    assert sha(path) == row['source_sha256'] == inputs[row['source']]
    assert not row['requires_strict_legacy_TW']
    body, start, end = extract(path.read_text('utf-8-sig'), row['test'].split('::')[-1], row['ignore_line'])
    assert not re.search(r'fs::write|File::create|Command::|TcpStream|TcpListener|CreateWindow|SendInput|waveOut', body)
    assert all(token in body for token in ['screen_test_support::compare', 'screen_test_support::retired', 'Sha256::digest'])
    bodies[row['test']] = dict(text=body, start_line=start, end_line=end)
helper = R / 'LiTo381-Modern/src/modern_lito/teleport_effect.rs'
assert sha(helper) == inputs[helper.relative_to(R).as_posix()]
helper_text = helper.read_text('utf-8-sig')
module_at = helper_text.index('pub(crate) mod screen_test_support {')
helpers = {}
for name in ['background', 'reference', 'compare', 'retired']:
    at = helper_text.index('fn ' + name + '(', module_at)
    line = helper_text[:at].count('\n') + 1
    body, start, end = extract(helper_text, name, line)
    assert not re.search(r'fs::write|File::create|Command::|TcpStream|CreateWindow|SendInput|waveOut', body)
    helpers[name] = dict(text=body, start_line=start, end_line=end)
out = V / 'native-current-library-haste-resource2' / ('plan-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True)
save(out / 'test-bodies.json', bodies)
save(out / 'screen-helpers.json', helpers)
record = dict(status='reviewed_two_haste_effect_resource_bodies_and_pure_CPU_helpers_no_execution', source_manifest_sha256=source,
              library=w['current_native_catalog_library']['run'], selected_tests=selected, names=names, expected_tests_per_profile=2,
              test_bodies=str(out / 'test-bodies.json'), test_bodies_sha256=sha(out / 'test-bodies.json'),
              screen_helpers=str(out / 'screen-helpers.json'), screen_helpers_sha256=sha(out / 'screen-helpers.json'),
              helper_source=str(helper), helper_source_sha256=sha(helper), Rust_function_parser_sha256=sha(parser_path),
              environment=dict(LITO_SHARED_RESOURCE_ROOT=r'D:\lineage開發\0-客戶端'),
              reviewed_prerequisites='Explicit read-only shared assets; complete SPR/source/encoded/RGBA bounds and independent six-frame RGBA goldens; pure Canvas complement reference, deterministic repeats and retirement; no HWND/device/JVM/TLS/SQL or output writer.',
              original_source_inputs=str(normal / 'compiled-inputs.json'), original_source_inputs_sha256=sha(normal / 'compiled-inputs.json'),
              plan_does_not_execute_tests=True, if_both_profiles_later_pass_additional_unique_ignored_tests=2,
              prior_actual_resource_optins=83, prior_remaining_ignored_unexecuted=405,
              gameplay_or_status_duration_1200_seconds_verified=False, native_skill43_enabled=False,
              new_sessions=0, new_casts=0, GUI_verified=False, physical_audio_verified=False, whole_goal_complete=False)
save(out / 'plan.json', record)
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(plan=str(out / 'plan.json'), sha256=sha(out / 'plan.json'), tests=2, executed=False), ensure_ascii=False))
