"""Inventory original assets using a pinned existing Debug library.

This inspects metadata and decoded members only. It does not execute a game
world, timing-sensitive scenario, candidate skill, normal build or audio device.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse
import ctypes
import hashlib
import json
import os
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')


ap = argparse.ArgumentParser()
ap.add_argument('--broad-cohort', type=Path, required=True)
ap.add_argument('--codec-qualification', type=Path, required=True)
a = ap.parse_args()
broad = a.broad_cohort.resolve(strict=True)
assert broad.is_relative_to(ROOT / '驗證紀錄/current/native-default111')
c = read(broad / 'cohort.json')
assert c['active_profile'] == 'Release' and c['status'] == 'executing_current_default111_profile'
debug = c['profiles']['Debug']
assert debug['qualified'] and debug['actual_runner_exit_code'] == 0
assert sha(debug['qualification']) == debug['qualification_sha256']
q = read(debug['qualification'])
assert q['actual_test_executables'] == 112 and q['counts']['failed'] == 0
codec = a.codec_qualification.resolve(strict=True)
assert codec.is_relative_to(ROOT / '驗證紀錄/current/native-attack-five-contract-inspection')
assert read(codec)['static_vectors'] == 12 and read(codec)['new_casts_executed'] == 0
release = []
for folder in (ROOT / '驗證紀錄/current/default-native-regression').glob('*-Release'):
    if not (folder / 'receipt.json').exists():
        continue
    receipt = read(folder / 'receipt.json')
    if receipt.get('normal_qualification') == c['normal_qualification'] and receipt.get('G06_qualification') == c['G06_qualification']:
        release.append((folder, receipt))
assert len(release) == 1 and release[0][1]['status'] == 'compiling_default_targets'
debug_run = Path(debug['run'])
inputs = read(debug_run / 'compiled-inputs.json')
assert len(inputs) == 1029
libraries = {}
native = set()
for line in (debug_run / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):
        continue
    row = json.loads(line)
    if row.get('reason') == 'compiler-artifact' and not row['profile']['test']:
        paths = [p for p in row['filenames'] if p.endswith('.rlib')]
        if paths and row['target']['name'] in ['lito_modern', 'anyhow', 'serde_json', 'sha2']:
            assert len(paths) == 1
            libraries[row['target']['name']] = dict(path=paths[0], sha256=sha(paths[0]))
    if row.get('reason') == 'build-script-executed':
        native.update(row.get('linked_paths', []))
assert set(libraries) == {'lito_modern', 'anyhow', 'serde_json', 'sha2'}
source = ROOT / '維護工具/candidates/native-attack-five/inspect_assets.rs'
stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out = ROOT / '驗證紀錄/current/native-attack-five-resource-inspection' / stamp
runtime = PROJECT / 'target/current/native-attack-five-resource-inspection' / stamp
out.mkdir(parents=True)
runtime.mkdir(parents=True)
(out / 'probe.rs').write_bytes(source.read_bytes())
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
product = runtime / 'asset-inventory.exe'
state = dict(status='prepared_readonly_asset_inventory', source_manifest_sha256=c['source_manifest_sha256'],
    current_Debug_broad_qualification=debug['qualification'], current_Debug_broad_qualification_sha256=debug['qualification_sha256'],
    existing_compiled_source_archive=str(debug_run / 'compiled-source.zip'),
    codec_qualification=str(codec), codec_qualification_sha256=sha(codec),
    observed_other_workload=dict(run=str(release[0][0]), status=release[0][1]['status']),
    libraries=libraries, probe_sha256=sha(source), owned_process=None, process_history=[],
    output=str(out), normal_product_build=False, production_source_changed=False,
    game_world_executed=False, new_sessions=0, new_casts=0, audio_device_opened=False,
    performance_acceptance_claimed=False, GUI_verified=False, whole_goal_complete=False)
env = dict(os.environ, TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'),
           PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8')


def check():
    assert sha(source) == sha(out / 'probe.rs') == state['probe_sha256']
    for name, digest in inputs.items():
        assert sha(ROOT / name) == digest, name
    for value in libraries.values():
        assert sha(value['path']) == value['sha256']
    assert sha(codec) == state['codec_qualification_sha256']


def execute(command, label):
    check()
    with (out / (label + '.stdout.txt')).open('wb') as stdout, (out / (label + '.stderr.txt')).open('wb') as stderr:
        child = subprocess.Popen(command, cwd=PROJECT, env=env, stdout=stdout, stderr=stderr,
                                 creationflags=subprocess.CREATE_NO_WINDOW)
        times = [wintypes.FILETIME() for _ in range(4)]
        get = ctypes.WinDLL('kernel32', use_last_error=True).GetProcessTimes
        get.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        get.restype = wintypes.BOOL
        assert get(wintypes.HANDLE(int(child._handle)), *[ctypes.byref(t) for t in times])
        ticks = times[0].dwLowDateTime | times[0].dwHighDateTime << 32
        job = dict(pid=child.pid, creation_utc=(datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)).isoformat(), command=command)
        state['process_history'].append(job)
        state['owned_process'] = job
        save(out / 'receipt.json', state)
        code = child.wait()
        job['exit_code'] = code
        state['owned_process'] = None
        save(out / 'receipt.json', state)
    check()
    assert code == 0, label + ' failed; retain the original evidence before repair'


save(out / 'receipt.json', state)
print('ASSET_INSPECTION=' + str(out), flush=True)
try:
    command = [str(RUST / 'rustc.exe'), str(out / 'probe.rs'), '--edition=2021', '--crate-name', 'lito_attack_five_asset_inventory',
        '--target', 'x86_64-pc-windows-msvc', '-C', 'opt-level=0', '-C', 'debuginfo=1',
        '-L', 'dependency=' + str(PROJECT / 'target/current/x86_64-pc-windows-msvc/debug/deps'),
        '-L', 'dependency=' + str(PROJECT / 'target/current/debug/deps'), '-o', str(product)]
    for name, info in libraries.items():
        command += ['--extern', name + '=' + info['path']]
    for value in sorted(native):
        command += ['-L', value]
    state['status'] = 'compiling_readonly_inventory_tool_from_existing_library'
    execute(command, 'rustc')
    state.update(status='reading_original_catalog_and_sprites_only', product_sha256=sha(product))
    execute([str(product), r'D:\lineage開發\0-客戶端', str(out / 'asset-inventory.json')], 'inventory')
    data = read(out / 'asset-inventory.json')
    assert data['catalog_sha256'] == '523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127'
    assert data['catalog_bytes'] == 4067417 and data['catalog_rows'] == 11297
    state.update(status='executed_readonly_original_asset_inventory', inventory_sha256=sha(out / 'asset-inventory.json'))
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    save(out / 'receipt.json', state)
print(json.dumps(dict(status=state['status'], output=str(out)), ensure_ascii=False), flush=True)
