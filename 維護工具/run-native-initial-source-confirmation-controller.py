"""Current source, real 75DA sessions, explicit TW/extended handoff and recovery.

This headless controller scope is separate from the required normal GUI/input
scope. The current startup job must finish and be independently qualified first.
The older full-library qualification is not promoted to this new source version.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, subprocess, zipfile, time, tomllib

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
now = lambda: datetime.now(timezone.utc).isoformat()
TARGETS = ['native_teleport_handoff', 'native_teleport_failure']
SCENARIOS = ['cross-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-expiry']
TOOLS = ['run-teleport-handoff-75DA.ps1', 'run-teleport-failure-75DA.ps1', 'verify-teleport-handoff-75DA.py',
         'verify-teleport-failure-75DA.py', 'verify-native-current-teleport-cancel-expiry-75DA.py',
         'qualify-native-initial-source-confirmation-controller-case.py', 'qualify-native-initial-source-confirmation-controller-cohort.py',
         'observe-native-initial-source-confirmation-controller-completion.py']

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('--related-run', required=True, type=Path)
ap.add_argument('--input-kind', choices=['catalog-normal'], default='catalog-normal')
ap.add_argument('--harness-source', type=Path, help='Owned phase-gated cross-endpoint fixture; explicitly pinned separately from product source')
a = ap.parse_args()
harness_override = None
if a.harness_source:
    path = a.harness_source.resolve(strict=True)
    assert path.is_relative_to(ROOT/'維護工具') and path.name=='NativeTeleportHarness.java'
    name = path.relative_to(ROOT/'維護工具').as_posix()
    TOOLS.append(name)
    harness_override = dict(path=str(path), validation_pin=name, sha256=sha(path), product_source_changed=False)
queue=read(ROOT/'工作佇列.json')
assert not queue.get('active_workloads') and not queue.get('source_and_corresponding_QA_frozen')
startup = a.related_run.resolve(strict=True)
assert startup.is_relative_to(ROOT/'驗證紀錄/current/native-initial-source-confirmation-normal')
c=read(startup/'cohort.json');assert c['status']=='executed_initial_source_confirmation_normal_related_both_profiles' and c['owned_process'] is None
assert len(c['process_history'])==10 and all(v['exit_code']==0 for v in c['process_history'])
qualification=startup/'qualified/qualified.json';q=read(qualification)
assert q['status']=='independently_qualified_initial_source_confirmation_normal_related_both_profiles' and q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_all_terminal']
assert q['source_manifest_sha256']==c['source_manifest_sha256'] and set(q['profiles'])=={'Debug','Release'} and q['passed_per_profile']==19
assert not q['full_broad_regression_verified'] and not q['full_library_verified'] and not q['source_overlays_used']
application=Path(c['application']);identity_manifest=application/'applied-source-inputs.json';pins=read(identity_manifest)
assert len(pins)==1047 and sha(identity_manifest)==c['source_manifest_sha256'] and pins==read(startup/'source-inputs.json')
inputs=read(startup/'compiled-inputs.json');assert len(inputs)==1048
retained=dict(path=q['compiled_source_archive'],sha256=q['compiled_source_archive_sha256'],files=inputs)
assert sha(retained['path'])==retained['sha256']
with zipfile.ZipFile(retained['path']) as z:
    assert set(z.namelist())==set(inputs)
    for name,digest in inputs.items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==digest
normal=dict(profiles={profile:info['normal'] for profile,info in c['profiles'].items()})
for profile,info in q['profiles'].items():
    assert info['normal']==normal['profiles'][profile] and info['normal']['sha256']==q['normal_mains'][profile]
    assert set(info['targets'])=={'native_login','native_services','native_teleport'}
    assert {name:item['counts'] for name,item in info['targets'].items()}=={'native_login':{'passed':3,'failed':0,'ignored':0},'native_services':{'passed':2,'failed':0,'ignored':0},'native_teleport':{'passed':14,'failed':0,'ignored':0}}
q['entry_sha256']=sha(ROOT/'啟動現代客戶端.ps1')
extra = dict(path='LiTo381-Modern/windows/app.manifest',sha256=inputs['LiTo381-Modern/windows/app.manifest'])
jar = PROJECT / 'target/current/server-reference/formal-server-75DA69B0.jar'
formal = ROOT.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
catalog = PROJECT / 'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
jar_sha = '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
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
    assert sha(qualification) == baseline_qualification_sha256
    assert sha(ROOT / '啟動現代客戶端.ps1') == q['entry_sha256']
    for info in normal['profiles'].values():
        assert sha(info['path']) == info['sha256']

baseline_qualification_sha256=sha(qualification)
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
assert release=={'opt-level':'z','lto':'thin','codegen-units':16,'strip':'symbols'}
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(ROOT)
check()
out = ROOT / '驗證紀錄/current/native-initial-source-confirmation-controller' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
kernel = ctypes.WinDLL('kernel32',use_last_error=True)
kernel.GetCurrentProcess.restype = wintypes.HANDLE
kernel.GetProcessTimes.argtypes = [wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
kernel.GetProcessTimes.restype = wintypes.BOOL
own_times = [wintypes.FILETIME() for _ in range(4)]
assert kernel.GetProcessTimes(kernel.GetCurrentProcess(),*[ctypes.byref(t) for t in own_times])
own_ticks = own_times[0].dwLowDateTime | own_times[0].dwHighDateTime << 32
save(out/'runner-identity.json',dict(ProcessId=os.getpid(),creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=own_ticks//10)).isoformat()))
(out / 'source-inputs.json').write_bytes(identity_manifest.read_bytes())
for name in TOOLS:
    destination=out/name;destination.parent.mkdir(parents=True,exist_ok=True)
    destination.write_bytes((ROOT / '維護工具' / name).read_bytes())
state = dict(status='prepared_current_G06_TW_extended_controller_scope', output=str(out),
             prerequisite_input_kind=a.input_kind, cross_endpoint_fixture_override=harness_override, release_configuration=release,
             source_files=1047, source_manifest_sha256=c['source_manifest_sha256'],
             additional_resource_input=extra, compiled_source_archive=retained,
             startup_prerequisite=str(startup), startup_qualification=str(qualification), startup_qualification_sha256=sha(qualification),
             full_current_library_qualified=False,
             normal_mains={profile:info['sha256'] for profile,info in normal['profiles'].items()}, reference_jar_sha256=jar_sha, common_catalog_sha256=catalog_sha,
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
            script = 'run-teleport-handoff-75DA.ps1' if scenario == 'cross-endpoint' else 'run-teleport-failure-75DA.ps1'
            family = 'native-teleport-handoff' if scenario == 'cross-endpoint' else 'native-teleport-failure'
            case_root = ROOT / '驗證紀錄/current' / family
            before = set(case_root.iterdir()) if case_root.exists() else set()
            child_env = env.copy()
            child_env['JAVA_TOOL_OPTIONS'] = java_options
            command = ['pwsh.exe', '-NoProfile', '-File', str(ROOT / '維護工具' / script),
                       '-TestBinary', products[target]['path'], '-ReferenceJar', str(jar), '-Scenario', scenario]
            if scenario=='cross-endpoint' and harness_override:
                command += ['-HarnessSource', harness_override['path']]
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
            validator = 'verify-teleport-handoff-75DA.py' if scenario == 'cross-endpoint' else 'verify-native-current-teleport-cancel-expiry-75DA.py' if scenario == 'cancel-expiry' else 'verify-teleport-failure-75DA.py'
            assert execute(['python', '-X', 'utf8', str(ROOT / '維護工具' / validator), str(case)], d / (scenario + '-comparison')) == 0
            assert execute(['python','-X','utf8',str(ROOT/'維護工具/qualify-native-initial-source-confirmation-controller-case.py'),str(out),'--profile',profile,'--scenario',scenario],d/(scenario+'-independent')) == 0
            check()
            print(profile + ' ' + scenario + ' real 75DA TW/extended controller case and comparison passed', flush=True)
    state.update(status='executed_current_both_profiles_G06_controller_TW_extended_six_cases', active_profile=None, active_scenario=None)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    state['finished_utc'] = now()
    persist()
print(json.dumps(dict(status=state['status'], output=str(out)), ensure_ascii=False), flush=True)
