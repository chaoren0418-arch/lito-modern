"""Retain default-feature related tests and the normal unique-entry main."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import shutil
import zipfile
root = Path(__file__).resolve().parents[1]
stage = root / '驗證紀錄/current/native-teleport-wire'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
ap = argparse.ArgumentParser()
ap.add_argument('--profile', choices=['Debug', 'Release'], required=True)
profile = ap.parse_args().profile
pins = read(stage / 'green-inputs-before.json')
assert len(pins) == 965
for name, expected in pins.items():
    assert sha(root / name) == expected, name
log_path = stage / f'{profile}-default-related-tests.jsonl'
log = log_path.read_text(encoding='utf-8-sig')
counts = [list(map(int, m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', log)]
assert counts == [[3, 0, 0], [9, 0, 0], [2, 0, 0], [8, 0, 0]], counts
messages = []
for line in log.splitlines():
    if line.startswith('{'):
        messages.append(json.loads(line))
assert any(d.get('reason') == 'build-finished' and d['success'] for d in messages)
tests = [d for d in messages if d.get('reason') == 'compiler-artifact' and d.get('executable') and d['profile'].get('test')]
assert {d['target']['name'] for d in tests} == {'native_login', 'native_network', 'native_services', 'native_teleport'}
assert len(tests) == 4 and all(d['features'] == [] for d in tests)
main = root / f'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/{profile.lower()}/LiToModern.exe'
entry_log = stage / f'{profile}-default-main-build.log'
assert entry_log.is_file() and str(main).replace('\\', '/') in entry_log.read_text(encoding='utf-8-sig').replace('\\', '/')
products = [{'kind': 'normal_main', 'path': str(main), 'sha256': sha(main)}]
archive = stage / f'{profile}-default-products.zip'
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as z:
    z.write(main, main.name)
    for d in tests:
        p = Path(d['executable']).resolve()
        assert p.is_relative_to(root / 'LiTo381-Modern/target/current')
        products.append({'kind': d['target']['name'], 'path': str(p), 'sha256': sha(p)})
        z.write(p, p.name)
with zipfile.ZipFile(archive) as z:
    for p in products:
        assert hashlib.sha256(z.read(Path(p['path']).name)).hexdigest().upper() == p['sha256']
validator = stage / 'validation-sources' / Path(__file__).name
validator.parent.mkdir(exist_ok=True)
shutil.copy2(Path(__file__), validator)
result = {'status': 'qualified_default_feature_related22_and_unique_entry_main', 'profile': profile,
          'source_files': 965, 'source_manifest_sha256': sha(stage / 'green-inputs-before.json'),
          'source_inputs_unchanged': True, 'features': [], 'normal_main': products[0],
          'unique_entry': str(root / '啟動現代客戶端.ps1'), 'entry_sha256': sha(root / '啟動現代客戶端.ps1'),
          'entry_build_log_sha256': sha(entry_log), 'related_tests_passed': 22, 'related_test_log_sha256': sha(log_path),
          'owner_dimensions_checked': 7, 'products': products, 'archive': str(archive), 'archive_sha256': sha(archive),
          'archive_readback_verified': True, 'GUI_verified': False, 'full_library_regression_verified': False,
          'whole_goal_complete': False}
(stage / f'{profile}-default-qualified.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'status': result['status'], 'profile': profile, 'passed': 22, 'main_sha256': products[0]['sha256']}, ensure_ascii=False))
