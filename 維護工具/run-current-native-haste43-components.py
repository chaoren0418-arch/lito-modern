"""Off-tree baseline/candidate components after the current broad job terminates.

The wrappers are deliberately not normal product PEs or live controller casts.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import ctypes
import hashlib
import json
import os
import re
import subprocess
import zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
P = R / 'LiTo381-Modern'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
save = lambda path, value: Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
w = read(R / '工作佇列.json')
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
source = w['current_product_identity']['source_manifest_sha256']
assert source == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
broad = w['current_native_catalog_broad_qualification']
assert broad['source_manifest_sha256'] == source and broad['recorded_original_owned_processes_all_terminal']
assert sha(broad['completion_observation']) == broad['completion_observation_sha256']
normal = Path(w['current_native_installed_resource_profile_normal_qualification']['run'])
inputs = read(normal / 'compiled-inputs.json')
prep_path = R / '維護工具/candidates/native-haste43-20261020-074819-989309/component-preparation-20261020-105958-213151/preparation.json'
prep = read(prep_path)
candidate_path = Path(prep['candidate'])
c = read(candidate_path)
assert sha(candidate_path) == prep['candidate_sha256'] == '2026A1527B23E1309D6D4237661229349BCCAA5BAE13D602576FA4E3B4B40A5F'
assert c['base_source_manifest_sha256'] == source and c['source_applied'] is False
rustc = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rustc.exe')
target = 'x86_64-pc-windows-msvc'
extern_pins = {}


def guard():
    assert len(inputs) == 1049 and all(sha(R / name) == digest for name, digest in inputs.items())
    assert all(sha(c['candidate_files'][name]) == digest for name, digest in c['after'].items())
    assert all(sha(info['path']) == info['sha256'] for info in prep['probe_sources'].values())
    assert all(sha(name) == digest for name, digest in prep['compatibility_test_sources'].items())
    assert all(sha(name) == info['sha256'] == sha(info['source']) for name, info in prep['fixtures'].items())
    assert all(sha(name) == digest for name, digest in extern_pins.items())


guard()
out = V / 'native-haste43-components' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
k = ctypes.WinDLL('kernel32', use_last_error=True)
k.GetCurrentProcess.restype = wintypes.HANDLE
k.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4


def birth(handle):
    values = [wintypes.FILETIME() for _ in range(4)]
    assert k.GetProcessTimes(handle, *[ctypes.byref(value) for value in values])
    ticks = values[0].dwLowDateTime | values[0].dwHighDateTime << 32
    return (datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)).isoformat()


state = dict(status='preparing_off_tree_haste43_components', base_source_manifest_sha256=source,
             normal_product_source_manifest_sha256=source,
             candidate=str(candidate_path), candidate_sha256=sha(candidate_path), preparation=str(prep_path), preparation_sha256=sha(prep_path),
             broad_completion=broad['completion_observation'], broad_completion_sha256=broad['completion_observation_sha256'],
             pipeline=dict(pid=os.getpid(), creation_utc=birth(k.GetCurrentProcess())), profiles={}, process_history=[], owned_process=None,
             normal_source_tree_modified=False, candidate_applied_to_product=False, candidate_included_as_off_tree_modules=True,
             actual_formal75_casts=0, normal_product_qualification_granted=False, GUI_verified=False, whole_goal_complete=False)
save(out / 'cohort.json', state)
print(json.dumps(dict(output=str(out)), ensure_ascii=False), flush=True)
env = {name: value for name, value in os.environ.items() if not name.startswith('LITO_')}
env.update(TEMP=str(P / 'target/current/scratch'), TMP=str(P / 'target/current/scratch'))


def execute(command, folder, child_env, expected, timeout):
    folder.mkdir(parents=True)
    guard()
    with (folder / 'stdout.log').open('wb') as stdout, (folder / 'stderr.log').open('wb') as stderr:
        child = subprocess.Popen(command, cwd=folder, env=child_env, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
        row = dict(pid=child.pid, creation_utc=birth(wintypes.HANDLE(int(child._handle))), command=command, exit_code=None)
        state['process_history'].append(row)
        state['owned_process'] = row
        save(out / 'cohort.json', state)
        try:
            child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=10)
            row['exact_owned_handle_timeout_cleanup'] = True
            raise
        finally:
            row.update(exit_code=child.returncode, finished_utc=datetime.now(timezone.utc).isoformat())
            state['owned_process'] = None
            save(out / 'cohort.json', state)
    assert child.returncode == expected, (str(folder), child.returncode, expected)
    guard()
    return row


try:
    for profile in ['Debug', 'Release']:
        broad_profile = broad['profiles'][profile]
        build = Path(broad_profile['run']) / 'build/stdout.log'
        messages = [json.loads(line) for line in build.read_text('utf-8-sig').splitlines() if line.startswith('{')]
        assert any(message.get('reason') == 'build-finished' and message['success'] for message in messages)
        kind = profile.lower()
        deps = P / 'target/current' / target / kind / 'deps'
        extern = {}
        for name in prep['required_extern_crates']:
            found = {str(Path(filename).resolve(strict=True)) for message in messages
                     if message.get('reason') == 'compiler-artifact' and message['target']['name'] == name
                     for filename in message['filenames'] if filename.endswith('.rlib') and Path(filename).parent == deps}
            assert len(found) == 1, (profile, name, found)
            extern[name] = found.pop()
        native_paths = sorted({path for message in messages if message.get('reason') == 'build-script-executed' for path in message['linked_paths']})
        extern_pins.update({path: sha(path) for path in extern.values()})
        work = out / profile
        (work / 'target/current/verification').mkdir(parents=True)
        child_env = dict(env, CARGO_MANIFEST_DIR=str(work))
        state['profiles'][profile] = dict(broad_build_log=str(build), broad_build_log_sha256=sha(build),
                                         extern={name: dict(path=path, sha256=sha(path)) for name, path in extern.items()}, native_link_paths=native_paths, modes={})
        state.update(status='compiling_off_tree_haste43_components', active_profile=profile)
        save(out / 'cohort.json', state)
        for mode in ['baseline', 'candidate']:
            exe = work / (mode + '.exe')
            command = [str(rustc), '--test', prep['probe_sources'][mode]['path'], '--edition=2021', '--target', target,
                       '--crate-name', 'haste43_' + kind + '_' + mode, '-o', str(exe),
                       '-L', 'dependency=' + str(deps), '-L', 'dependency=' + str(P / 'target/current' / kind / 'deps')]
            command += ['-C', 'opt-level=0', '-C', 'debuginfo=2'] if profile == 'Debug' else ['-C', 'opt-level=z', '-C', 'lto=thin', '-C', 'codegen-units=16', '-C', 'strip=symbols']
            for name, path in extern.items():
                command += ['--extern', name + '=' + path]
            for path in native_paths:
                command += ['-L', path]
            execute(command, work / (mode + '-compile'), child_env, 0, 300)
            command = [str(exe)]
            if mode == 'baseline':
                command += prep['baseline_selected_expected_failures'] + ['--exact']
            command += ['--test-threads=1']
            state.update(status='executing_off_tree_haste43_' + mode, active_mode=mode)
            execute(command, work / (mode + '-run'), child_env, 101 if mode == 'baseline' else 0, 600)
            log = (work / (mode + '-run/stdout.log')).read_text('utf-8-sig')
            results = re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$', log, re.M)
            if mode == 'baseline':
                assert len(results) == 3 and {name for name, _ in results} == set(prep['baseline_selected_expected_failures'])
                assert all(result == 'FAILED' for _, result in results)
                assert '0 passed; 3 failed; 0 ignored; 0 measured; 17 filtered out' in log
            else:
                assert len(results) == 20 and all(result == 'ok' for _, result in results)
                assert '20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in log
            state['profiles'][profile]['modes'][mode] = dict(executable=str(exe), sha256=sha(exe), results=dict(results), expected_Rust_exit=101 if mode == 'baseline' else 0)
            save(out / 'cohort.json', state)
            print(json.dumps(dict(profile=profile, mode=mode, tests=len(results), baseline_failures_expected=mode == 'baseline'), ensure_ascii=False), flush=True)
    state.update(status='executed_off_tree_haste43_baseline_and_candidate_components_both_profiles', active_mode=None, active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    guard()
    state['finished_utc'] = datetime.now(timezone.utc).isoformat()
    save(out / 'cohort.json', state)
print(json.dumps(dict(output=str(out), status=state['status'], candidate_applied=False, formal75_casts=0), ensure_ascii=False), flush=True)
