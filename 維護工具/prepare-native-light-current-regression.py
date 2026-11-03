"""Prepare version-bound regression drivers; preserve all older validation tools."""
import ast
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
MAINT = ROOT / '維護工具'
STAGE = ROOT / '驗證紀錄/current/native-light-spell/current-regression'
STAGE.mkdir(exist_ok=True)
assert not (STAGE / 'preparation.json').exists(), 'Preserve completed preparation'
normal_path = ROOT / '驗證紀錄/current/native-light-spell/normal/qualified.json'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
normal = read(normal_path)
pins = read(normal_path.parent / 'source-inputs.json')
assert len(pins) == normal['source_files'] == 970
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
for image in normal['profiles'].values():
    assert sha(Path(image['path'])) == image['sha256']
tools = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
env = os.environ.copy()
env.update(RUSTC=str(tools / 'rustc.exe'), RUSTDOC=str(tools / 'rustdoc.exe'),
           TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'))
env['PATH'] = str(tools) + os.pathsep + env['PATH']
metadata = subprocess.run([str(tools / 'cargo.exe'), 'metadata', '--no-deps', '--locked', '--offline', '--format-version', '1'],
                          cwd=PROJECT, env=env, check=True, capture_output=True)
(STAGE / 'metadata.json').write_bytes(metadata.stdout)
package = next(p for p in json.loads(metadata.stdout)['packages'] if Path(p['manifest_path']).resolve() == PROJECT / 'Cargo.toml')
targets = [t for t in package['targets'] if 'test' in t['kind']]
defaults = sorted(t['name'] for t in targets if not t.get('required-features'))
excluded = {t['name']: t['required-features'] for t in targets if t.get('required-features')}
assert len(defaults) == 91 and 'native_light_spell' in defaults
assert excluded == {'native_teleport_controller': ['teleport-wire-test']}
originals = {}
prepared = {}

def clone(source, destination, replacements):
    original = MAINT / source
    text = original.read_text(encoding='utf-8-sig')
    originals[source] = sha(original)
    for before, after in replacements:
        assert before in text, (source, before)
        text = text.replace(before, after)
    path = MAINT / destination
    assert not path.exists(), destination
    path.write_text(text, encoding='utf-8')
    if destination.endswith('.py'):
        ast.parse(text)
    prepared[destination] = sha(path)

clone('run-default-native-regression.py', 'run-native-light-default-regression.py', [
    ("normal['source_files'] == 968", "normal['source_files'] == 970"),
    ('len(suites) == 90', "len(suites) == 91 and 'native_light_spell' in suites"),
    ('verified_current_default90_integration_targets_and_main_units', 'verified_current_default91_integration_targets_and_main_units'),
    (", errors='replace'", ''),
    ("env = os.environ.copy()", "# Retain exact compiled inputs before any compilation or possible failure.\nsource_archive = output / 'compiled-source.zip'\nwith zipfile.ZipFile(source_archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:\n    for name, expected in pins.items():\n        assert sha(ROOT / name) == expected\n        z.write(ROOT / name, name)\nwith zipfile.ZipFile(source_archive) as z:\n    assert set(z.namelist()) == set(pins)\n    for name, expected in pins.items():\n        assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected\nreceipt['compiled_source_archive_sha256'] = sha(source_archive)\nreceipt['compiled_source_archive_readback_verified'] = True\npersist()\n\nenv = os.environ.copy()"),
])
clone('qualify-default-native-regression.py', 'qualify-native-light-default-regression.py', [
    ("r['source_files']==968", "r['source_files']==970"),
    ('len(suites)==90', "len(suites)==91 and 'native_light_spell' in suites"),
    ("'integration_targets':90,'products_retained':91", "'integration_targets':91,'products_retained':92"),
    ('verified_current_default90_integration_targets_and_main_units', 'verified_current_default91_integration_targets_and_main_units'),
    (",errors='replace'", ''),
    ("assert r['compile_exit_code']==0 and r['failed']==0", "assert r['compile_exit_code']==0 and r['failed']==0\nassert r['compiled_source_archive_readback_verified'] and sha(run/'compiled-source.zip')==r['compiled_source_archive_sha256']"),
])
clone('run-current-teleport-GUI-revalidation.ps1', 'run-native-light-current-teleport-GUI.ps1', [
    ('$rust.integration_targets -ne 90 -or $rust.products_retained -ne 91', '$rust.integration_targets -ne 91 -or $rust.products_retained -ne 92'),
    ('qualify-native-teleport-current-case.py', 'qualify-native-light-teleport-current-case.py'),
])
clone('qualify-native-teleport-current-case.py', 'qualify-native-light-teleport-current-case.py', [
    ("rust['integration_targets']==90 and rust['products_retained']==91", "rust['integration_targets']==91 and rust['products_retained']==92"),
    ('qualify-default-native-regression.py', 'qualify-native-light-default-regression.py'),
    ('verified_current_default90_integration_targets_and_main_units', 'verified_current_default91_integration_targets_and_main_units'),
    ("len(rust['per_target_pins'])==91", "len(rust['per_target_pins'])==92"),
])
clone('qualify-native-teleport-current-cohort.py', 'qualify-native-light-teleport-current-cohort.py', [
    ("n['source_files']==968", "n['source_files']==970"),
    ("rust['integration_targets']==90", "rust['integration_targets']==91"),
    ("q['source_files']==968", "q['source_files']==970"),
    ("'source_files':968", "'source_files':970"),
    ('run-current-teleport-GUI-revalidation.ps1', 'run-native-light-current-teleport-GUI.ps1'),
    ('qualify-native-teleport-current-case.py', 'qualify-native-light-teleport-current-case.py'),
])
clone('run-native-higher-heal-default90-followup.ps1', 'run-native-light-default91-followup.ps1', [
    ('native-shared-higher-heals/peer-close-fix', 'native-light-spell/current-regression'),
    ("$normal=Join-Path $stage 'normal/qualified.json'", "$normal=Join-Path $taskRoot '驗證紀錄/current/native-light-spell/normal/qualified.json'"),
    ('default90', 'default91'),
    ('run-default-native-regression.py', 'run-native-light-default-regression.py'),
    ('qualify-default-native-regression.py', 'qualify-native-light-default-regression.py'),
])
clone('run-native-higher-heal-TW14-followup.ps1', 'run-native-light-TW14-followup.ps1', [
    ('native-shared-higher-heals/peer-close-fix', 'native-light-spell/current-regression'),
    ("$normal=Join-Path $stage 'normal/qualified.json'", "$normal=Join-Path $taskRoot '驗證紀錄/current/native-light-spell/normal/qualified.json'"),
    ('default90', 'default91'),
    ('run-current-teleport-GUI-revalidation.ps1', 'run-native-light-current-teleport-GUI.ps1'),
    ('qualify-native-teleport-current-cohort.py', 'qualify-native-light-teleport-current-cohort.py'),
])
for name, expected in originals.items():
    assert sha(MAINT / name) == expected
plan = {'status': 'prepared_not_executed', 'normal_qualification': str(normal_path),
        'normal_qualification_sha256': sha(normal_path), 'source_files': len(pins),
        'source_manifest_sha256': sha(normal_path.parent / 'source-inputs.json'),
        'normal_mains': {p: item['sha256'] for p, item in normal['profiles'].items()},
        'integration_targets': defaults, 'default_integration_targets': len(defaults),
        'products_per_profile': len(defaults) + 1, 'excluded_required_feature_targets': excluded,
        'original_tools_preserved': originals, 'prepared_tools': prepared,
        'TW14_requires_both_default_profiles_qualified': True,
        'new_game_sessions': 0, 'whole_goal_complete': False}
(STAGE / 'preparation.json').write_text(json.dumps(plan, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'status': plan['status'], 'default_integration_targets': len(defaults),
                  'products_per_profile': len(defaults) + 1, 'source_files': len(pins)}, ensure_ascii=False))
