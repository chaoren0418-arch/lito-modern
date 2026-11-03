"""Current source, real C02 sessions, explicit TW/extended handoff and recovery.

This headless controller scope is separate from the required normal GUI/input
scope. The existing full-library job must finish before compilation begins.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
now = lambda: datetime.now(timezone.utc).isoformat()
TARGETS = ['native_teleport_handoff', 'native_teleport_failure']
SCENARIOS = ['cross-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-expiry']
TOOLS = ['run-teleport-handoff.ps1', 'run-teleport-failure.ps1', 'verify-teleport-handoff.py',
         'verify-teleport-failure.py', 'verify-native-current-teleport-cancel-expiry.py']

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('--library-run', required=True, type=Path)
a = ap.parse_args()
library = a.library_run.resolve(strict=True)
assert library.is_relative_to(ROOT / '驗證紀錄/current/native-teleport-wire')
c = read(library / 'cohort.json')
assert c['status'] == 'executed_corrected_TX_retirement_and_both_full_default_libraries' and c['owned_process'] is None
for profile in ['Debug', 'Release']:
    q = read(library / profile / 'full-default-qualified/qualified.json')
    assert (q['passed'], q['failed'], q['ignored']) == (1935, 0, 473)
    assert q['source_manifest_sha256'] == c['source_manifest_sha256']
pins = read(library / 'source-inputs.json')
assert len(pins) == 1021
normal = read(library / 'normal-invariance.json')
extra = normal['additional_normal_resource_input']
jar = PROJECT / 'target/current/server-reference/formal-server-C02AAAE8.jar'
formal = ROOT.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
catalog = PROJECT / 'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
jar_sha = 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
catalog_sha = 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
qa = {name: sha(ROOT / '維護工具' / name) for name in TOOLS}
assert not {k: v for k, v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'JAVA_TOOL_OPTIONS', 'JDK_JAVA_OPTIONS'])}

def check():
    for name, digest in pins.items():
        assert sha(ROOT / name) == digest, name
    assert sha(ROOT / extra['path']) == extra['sha256']
    assert sha(jar) == sha(formal) == jar_sha
    assert sha(catalog) == catalog_sha
    for name, digest in qa.items():
        assert sha(ROOT / '維護工具' / name) == digest, name
    for info in normal['profiles'].values():
        assert sha(info['path']) == info['sha256']

check()
out = ROOT / '驗證紀錄/current/native-G06-controller' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
(out / 'source-inputs.json').write_bytes((library / 'source-inputs.json').read_bytes())
for name in TOOLS:
    (out / name).write_bytes((ROOT / '維護工具' / name).read_bytes())
state = dict(status='prepared_current_G06_TW_extended_controller_scope', output=str(out),
             source_files=1021, source_manifest_sha256=c['source_manifest_sha256'],
             additional_resource_input=extra, source_retention=c['source_retention'],
             library_prerequisite=str(library), normal_invariance_proof=str(library / 'normal-invariance.json'),
             normal_mains=c['normal_mains'], reference_jar_sha256=jar_sha, common_catalog_sha256=catalog_sha,
             resource_profile='tw13081901-v1', extended_item_details=True,
             scenarios=SCENARIOS, validation_pins=qa, profiles={}, owned_process=None, process_history=[],
             GUI_verified=False, physical_input_verified=False, same_endpoint_verified=False,
             whole_G06_complete=False, whole_goal_complete=False, started_utc=now())
persist = lambda: save(out / 'cohort.json', state)
env = os.environ.copy()
env.update(RUSTC=str(RUST / 'rustc.exe'), RUSTDOC=str(RUST / 'rustdoc.exe'),
           PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8',
           TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'))
env['PATH'] = str(RUST) + os.pathsep + env['PATH']
java_options = '-Dfile.encoding=UTF-8 -Dlito.resourceProfile=tw13081901-v1 -Dlito.extendedItemDetails=true -Dlito.commonCatalog=' + str(catalog)
state['JAVA_TOOL_OPTIONS_for_owned_children_only'] = java_options

def execute(command, folder, child_env=None):
    folder.mkdir()
    with (folder / 'stdout.log').open('wb') as stdout, (folder / 'stderr.log').open('wb') as stderr:
        child = subprocess.Popen(command, cwd=PROJECT, env=child_env or env, stdout=stdout, stderr=stderr,
                                 creationflags=subprocess.CREATE_NO_WINDOW)
        times = [wintypes.FILETIME() for _ in range(4)]
        call = ctypes.WinDLL('kernel32', use_last_error=True).GetProcessTimes
        call.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        call.restype = wintypes.BOOL
        assert call(wintypes.HANDLE(int(child._handle)), *[ctypes.byref(t) for t in times])
        ticks = times[0].dwLowDateTime | (times[0].dwHighDateTime << 32)
        created = datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)
        item = dict(pid=child.pid, creation_utc=created.isoformat(), command=command, log_directory=str(folder))
        state['owned_process'] = item
        state['process_history'].append(item)
        persist()
        code = child.wait()
        item.update(exit_code=code, finished_utc=now())
        state['owned_process'] = None
        persist()
    return code

persist()
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
try:
    for profile in ['Debug', 'Release']:
        check()
        state.update(status='compiling_current_G06_controller_targets', active_profile=profile)
        d = out / profile
        d.mkdir()
        state['profiles'][profile] = dict(directory=str(d), cases={})
        command = [str(RUST / 'cargo.exe'), 'test', '--no-run', '--locked', '--offline', '-j1',
                   '--target', 'x86_64-pc-windows-msvc', '--target-dir', 'target/current', '--message-format=json']
        if profile == 'Release':
            command.append('--release')
        for target in TARGETS:
            command += ['--test', target]
        assert execute(command, d / 'build') == 0, 'Preserve compiler failure.'
        check()
        messages = [json.loads(line) for line in (d / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
        assert any(m.get('reason') == 'build-finished' and m['success'] for m in messages)
        artifacts = [m for m in messages if m.get('reason') == 'compiler-artifact' and m.get('executable') and m['profile']['test']]
        assert {m['target']['name'] for m in artifacts} == set(TARGETS) and len(artifacts) == 2
        products = {}
        for m in artifacts:
            assert m['features'] == []
            path = Path(m['executable']).resolve(strict=True)
            assert path.is_relative_to(PROJECT / 'target/current')
            products[m['target']['name']] = dict(path=str(path), sha256=sha(path), compiler_profile=m['profile'])
        save(d / 'products.json', products)
        with zipfile.ZipFile(d / 'test-products.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
            for name, product in products.items():
                z.write(product['path'], name + '.exe')
        with zipfile.ZipFile(d / 'test-products.zip') as z:
            for name, product in products.items():
                with z.open(name + '.exe') as f:
                    assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == product['sha256']
        state['profiles'][profile].update(products=products, test_archive_sha256=sha(d / 'test-products.zip'))
        persist()
        for scenario in SCENARIOS:
            check()
            state.update(status='executing_current_G06_TW_extended_case', active_scenario=scenario)
            target = 'native_teleport_handoff' if scenario == 'cross-endpoint' else 'native_teleport_failure'
            script = 'run-teleport-handoff.ps1' if scenario == 'cross-endpoint' else 'run-teleport-failure.ps1'
            family = 'native-teleport-handoff' if scenario == 'cross-endpoint' else 'native-teleport-failure'
            case_root = ROOT / '驗證紀錄/current' / family
            before = set(case_root.iterdir()) if case_root.exists() else set()
            child_env = env.copy()
            child_env['JAVA_TOOL_OPTIONS'] = java_options
            command = ['pwsh.exe', '-NoProfile', '-File', str(ROOT / '維護工具' / script),
                       '-TestBinary', products[target]['path'], '-ReferenceJar', str(jar), '-Scenario', scenario]
            code = execute(command, d / scenario, child_env)
            created = set(case_root.iterdir()) - before
            assert len(created) == 1, ('case identity', created)
            case = created.pop()
            state['profiles'][profile]['cases'][scenario] = dict(directory=str(case), exit_code=code, product_sha256=products[target]['sha256'])
            persist()
            assert code == 0, 'Preserve runtime failure, including partial authority and cleanup.'
            receipt = read(case / 'receipt.json')
            assert receipt['jar_sha256'] == jar_sha and receipt['test_binary_sha256'] == products[target]['sha256']
            raw_error = (case / 'server.stderr.txt').read_bytes()
            try:
                server_error = raw_error.decode('utf-8-sig')
                error_encoding = 'utf-8-sig'
            except UnicodeDecodeError:
                server_error = raw_error.decode('cp950')
                assert server_error.encode('cp950') == raw_error
                error_encoding = 'cp950_strict_roundtrip'
            assert 'Picked up JAVA_TOOL_OPTIONS: ' + java_options in server_error
            state['profiles'][profile]['cases'][scenario]['server_stderr_encoding'] = error_encoding
            validator = 'verify-teleport-handoff.py' if scenario == 'cross-endpoint' else 'verify-native-current-teleport-cancel-expiry.py' if scenario == 'cancel-expiry' else 'verify-teleport-failure.py'
            assert execute(['python', '-X', 'utf8', str(ROOT / '維護工具' / validator), str(case)], d / (scenario + '-comparison')) == 0
            check()
            print(profile + ' ' + scenario + ' real C02 TW/extended controller case and comparison passed', flush=True)
    state.update(status='executed_current_both_profiles_G06_controller_TW_extended_six_cases', active_profile=None, active_scenario=None)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    state['finished_utc'] = now()
    persist()
print(json.dumps(dict(status=state['status'], output=str(out)), ensure_ascii=False), flush=True)
