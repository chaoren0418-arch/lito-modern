"""Two additionally reviewed Haste effect resource bodies, using existing qualified library PEs.

No compiler, Java peer, audio device, GUI, or product change is involved.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse
import ctypes
import hashlib
import json
import os
import re
import subprocess

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
P = R / 'LiTo381-Modern'
SOURCE = 'E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
save = lambda path, value: Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
names = ['modern_lito::haste_effect::tests::real_shared_haste43_effect_frames_and_terminal', 'modern_lito::greater_haste_effect::tests::real_shared_greater_haste_effect_frames_and_terminal']
parser = argparse.ArgumentParser()
parser.add_argument('--plan', type=Path)
args = parser.parse_args()
w = read(R / '工作佇列.json')
assert w['current_product_identity']['source_manifest_sha256'] == SOURCE
library = Path(w['current_native_catalog_library']['run'])
completion_path = library / 'completion-observation.json'
completion = read(completion_path)
assert completion['source_manifest_sha256'] == SOURCE and completion['observed_terminal_exit_code'] == 0
assert completion['recorded_original_owned_processes_all_terminal']
inputs = read(library / 'compiled-inputs.json')
assert len(inputs) == 1049


def guard():
    assert all(sha(R / name) == digest for name, digest in inputs.items())
    for profile in ['Debug', 'Release']:
        retained = read(library / profile / 'product-retention.json')
        assert sha(retained['product']) == retained['product_sha256'] == completion['profiles'][profile]['actual_test_sha256']
        assert sha(retained['archive']) == retained['archive_sha256']


guard()
remaining_path = Path(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-library-remaining-optin-review\E0E6409E-20261020-120830-464631\review.json")
remaining = read(remaining_path)
assert remaining['source_manifest_sha256'] == SOURCE and remaining['remaining_exact_tests'] == 407
selected = [row for row in remaining['tests'] if row['test'] in names]
assert len(selected) == len(set(names)) == 2
for row in selected:
    assert sha(R / row['source']) == row['source_sha256'] == inputs[row['source']]
    assert not row['requires_strict_legacy_TW']


assert args.plan is not None
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
plan_path = args.plan.resolve(strict=True)
assert plan_path.is_relative_to(V / 'native-current-library-haste-resource2')
plan = read(plan_path)
assert plan['source_manifest_sha256'] == SOURCE and plan['names'] == names
assert sha(plan['test_bodies']) == plan['test_bodies_sha256'] and plan['runner_sha256'] == sha(__file__)
assert plan['library_completion_sha256'] == sha(completion_path)
out = V / 'native-current-library-haste-resource2' / ('run-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
(out / 'plan.json').write_bytes(plan_path.read_bytes())
k = ctypes.WinDLL('kernel32', use_last_error=True)
k.GetCurrentProcess.restype = wintypes.HANDLE
k.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4


def birth(handle):
    values = [wintypes.FILETIME() for _ in range(4)]
    assert k.GetProcessTimes(handle, *[ctypes.byref(value) for value in values])
    ticks = values[0].dwLowDateTime | values[0].dwHighDateTime << 32
    return (datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)).isoformat()


def metadata():
    root = Path(plan['environment']['LITO_SHARED_RESOURCE_ROOT'])
    paths = [path for path in root.rglob('*') if path.is_file()]
    assert all(not (getattr(path.lstat(), 'st_file_attributes', 0) & 0x400) for path in paths)
    return {str(path): [path.stat().st_size, path.stat().st_mtime_ns] for path in paths}


before_resources = metadata()
save(out / 'resource-metadata-before.json', before_resources)
state = dict(status='prepared_current_two_shared_haste_resource_tests', source_manifest_sha256=SOURCE,
             library=str(library), library_completion_sha256=sha(completion_path), plan=str(plan_path), plan_sha256=sha(plan_path),
             pipeline=dict(pid=os.getpid(), creation_utc=birth(k.GetCurrentProcess())), profiles={}, owned_process=None,
             expected_tests_per_profile=2, new_sessions=0, GUI_verified=False, audio_device_verified=False, whole_goal_complete=False)
save(out / 'cohort.json', state)
print(json.dumps(dict(output=str(out)), ensure_ascii=False), flush=True)
env = {name: value for name, value in os.environ.items() if not name.startswith('LITO_')}
env.update(plan['environment'], TEMP=str(P / 'target/current/scratch'), TMP=str(P / 'target/current/scratch'))
try:
    for profile in ['Debug', 'Release']:
        guard()
        retained = read(library / profile / 'product-retention.json')
        target = out / profile
        target.mkdir()
        command = [retained['product'], *names, '--exact', '--ignored', '--test-threads=1', '--nocapture']
        query = subprocess.run([retained['product'], *names, '--exact', '--ignored', '--list'], cwd=target, env=env, capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
        assert query.returncode == 0 and not query.stderr
        assert {line[:-6] for line in query.stdout.decode('utf-8-sig').splitlines() if line.endswith(': test')} == set(names)
        (target / 'selection.txt').write_bytes(query.stdout)
        with (target / 'tests.log').open('wb') as stdout, (target / 'stderr.log').open('wb') as stderr:
            child = subprocess.Popen(command, cwd=target, env=env, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
            row = dict(pid=child.pid, creation_utc=birth(wintypes.HANDLE(int(child._handle))), command=command,
                       product_sha256=sha(retained['product']), exit_code=None)
            state['profiles'][profile] = row
            state.update(status='running_current_two_shared_haste_resource_tests', active_profile=profile, owned_process=row)
            save(out / 'cohort.json', state)
            try:
                code = child.wait(timeout=300)
            except subprocess.TimeoutExpired:
                child.kill()  # exact original Popen handle, only this owned test
                child.wait(timeout=10)
                row['original_owned_handle_timeout_cleanup'] = True
                raise
            finally:
                row.update(exit_code=child.returncode, finished_utc=datetime.now(timezone.utc).isoformat())
                state['owned_process'] = None
                save(out / 'cohort.json', state)
        log = (target / 'tests.log').read_text('utf-8-sig')
        results = re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$', log, re.M)
        assert len(results) == 2 and {name for name, _ in results} == set(names) and all(value == 'ok' for _, value in results)
        assert '2 passed; 0 failed; 0 ignored; 0 measured; 2460 filtered out' in log and code == 0
        diagnostic = (target / 'stderr.log').read_text('utf-8-sig')
        assert 'HASTE43 sources=' in diagnostic and 'GREATER_HASTE sources=' in diagnostic
        row.update(passed=2, failed=0, ignored=0, filtered_out=2460, stdout_sha256=sha(target / 'tests.log'), stderr_sha256=sha(target / 'stderr.log'))
        assert metadata() == before_resources
        guard()
        save(out / 'cohort.json', state)
        print(json.dumps(dict(profile=profile, passed=2, failed=0), ensure_ascii=False), flush=True)
    state.update(status='executed_current_two_haste_resource_tests_both_profiles', active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    state.update(resource_metadata_unchanged=metadata() == before_resources, finished_utc=datetime.now(timezone.utc).isoformat())
    save(out / 'cohort.json', state)
    guard()
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
