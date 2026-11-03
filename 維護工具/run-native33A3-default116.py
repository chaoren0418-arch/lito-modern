"""Build current112 default integrations plus main after the current G06 controller cohort."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
import re
import subprocess
import zipfile
import time

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
STAGE = ROOT / '驗證紀錄/current/native-teleport-wire'
TOOLS = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
now = lambda: datetime.now(timezone.utc).isoformat()

def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest().upper()

def save(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

def creation_time(process):
    values = [wintypes.FILETIME() for _ in range(4)]
    call = ctypes.WinDLL('kernel32', use_last_error=True).GetProcessTimes
    call.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    call.restype = wintypes.BOOL
    assert call(wintypes.HANDLE(int(process._handle)), *[ctypes.byref(v) for v in values])
    ticks = (values[0].dwHighDateTime << 32) | values[0].dwLowDateTime
    return datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)

def stop_owned(process, args, created):
    if process.poll() is not None:
        return
    script = ("[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);"
              f"$p=Get-CimInstance Win32_Process -Filter 'ProcessId={process.pid}';"
              "if($p){[pscustomobject]@{pid=$p.ProcessId;path=$p.ExecutablePath;"
              "created=$p.CreationDate.ToUniversalTime().ToString('o');command=$p.CommandLine}|ConvertTo-Json -Compress}")
    encoded = __import__('base64').b64encode(script.encode('utf-16-le')).decode('ascii')
    result = subprocess.run(['powershell.exe', '-NoProfile', '-EncodedCommand', encoded],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NO_WINDOW)
    assert result.returncode == 0
    actual = json.loads(result.stdout.decode('utf-8-sig'))
    assert actual['pid'] == process.pid
    assert Path(actual['path']).resolve() == Path(args[0]).resolve()
    assert abs((datetime.fromisoformat(actual['created']) - created).total_seconds()) < .001
    assert actual['command'].strip() == subprocess.list2cmdline(args)
    process.kill()  # Popen retains the handle of this exact verified process.
    process.wait(timeout=10)

ap = argparse.ArgumentParser()
ap.add_argument('--profile', choices=['Debug', 'Release'], default='Debug')
ap.add_argument('--normal-qualification', type=Path, required=True)
ap.add_argument('--G06-qualification', type=Path, required=True)
ap.add_argument('--normal-kind', choices=['ability-finite-normal'], default='ability-finite-normal')
ap.add_argument('--preflight-only', action='store_true')
args = ap.parse_args()
profile = args.profile
normal_path = args.normal_qualification.resolve(strict=True)
assert normal_path.is_relative_to(ROOT / '驗證紀錄/current')
normal = read(normal_path)
assert normal_path.is_relative_to(ROOT/'驗證紀錄/current/native-ability-finite-normal')
assert normal['status']=='independently_qualified_ability_finite_normal_related_scope' and normal['profile']==profile
assert normal['passed']==31 and normal['failed']==0 and normal['default_ignored']==2 and normal['recorded_owned_all_terminal']
startup=normal_path.parent.parent
startup_receipt=read(startup/'cohort.json')
assert startup_receipt['owned_process'] is None
identity=Path(startup_receipt['application'])/'applied-source-inputs.json';assert sha(identity)==normal['source_manifest_sha256']=='33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21'
pins=read(identity);assert len(pins)==normal['source_files']==1047 and pins==read(startup/'source-inputs.json')
normal['entry_sha256']=startup_receipt['entry_sha256']
normal_products={selected:info['normal'] for selected,info in startup_receipt['profiles'].items()}
for selected,product in normal_products.items():
    qualification=read(next(startup.glob(selected+'-qualified-*/qualified.json')))
    assert qualification['source_manifest_sha256']==normal['source_manifest_sha256'] and qualification['passed']==31 and qualification['failed']==0
    assert product['sha256']==qualification['normal']['sha256'] and product['unique_entry_exit_code']==0
completion_path=args.G06_qualification.resolve(strict=True)
assert completion_path.is_relative_to(ROOT/'驗證紀錄/current/native1047-G06-controller')
completion=read(completion_path)
assert completion['status']=='independently_qualified_both_current_G06_controller_profiles_six_cases_each'
assert completion['observed_terminal_exit_code']==0 and completion['actual_formal_sessions']==96 and completion['source_manifest_sha256']==normal['source_manifest_sha256']
assert completion['normal_mains']=={selected:info['sha256'] for selected,info in normal_products.items()}
all_inputs=read(startup/'compiled-inputs.json');assert len(all_inputs)==1048
compile_fixtures={name:digest for name,digest in all_inputs.items() if name not in pins}
assert compile_fixtures=={'LiTo381-Modern/windows/app.manifest':startup_receipt['compiled_source_archive']['files']['LiTo381-Modern/windows/app.manifest']}
compiled_inputs=pins|compile_fixtures;assert compiled_inputs==all_inputs
assert not {k: v for k, v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS'])}

def check_inputs():
    for name, expected in compiled_inputs.items():
        assert sha(ROOT / name) == expected, ('source drift', name)
    assert sha(ROOT/'啟動現代客戶端.ps1')==normal['entry_sha256']

def mains():
    return {p: sha(Path(v['path'])) for p, v in normal_products.items()}

if args.preflight_only:
    check_inputs()
    expected={p:v['sha256'] for p,v in normal_products.items()};assert mains()==expected
    destination=ROOT/'驗證紀錄/current/native33A3-default116-preflight'/(datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+profile)
    destination.mkdir(parents=True)
    save(destination/'preflight.json',dict(status='current_source_default116_prerequisites_verified_only',profile=profile,source_manifest_sha256=normal['source_manifest_sha256'],source_files=len(pins),compiled_input_files=len(compiled_inputs),normal_mains=expected,normal_qualification=str(normal_path),normal_qualification_sha256=sha(normal_path),G06_qualification=str(completion_path),G06_qualification_sha256=sha(completion_path),broad_tests_executed=False,new_sessions=0,GUI_verified=False,whole_goal_complete=False))
    (destination/'runner.py').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(dict(preflight=str(destination),profile=profile,broad_tests_executed=False),ensure_ascii=False),flush=True)
    raise SystemExit(0)
from native33A3_broad_gate import require_idle_runtime
require_idle_runtime(ROOT)
quote=str(ROOT).replace("'","''")
probe="$r='"+quote+"';$p=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.Name -eq 'LiToModern.exe' -and $_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase))});if($p.Count){exit 1}"
assert subprocess.run(['pwsh','-NoProfile','-Command',probe],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW).returncode==0,'Owned GUI/server workload must be idle before broad regression'
check_inputs()
expected_mains = {p: v['sha256'] for p, v in normal_products.items()}
assert mains() == expected_mains
output = ROOT / '驗證紀錄/current/default-native-regression' / (datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')[:-3] + '-' + profile)
output.mkdir(parents=True, exist_ok=False)
(output / 'source-inputs.json').write_bytes(identity.read_bytes())
save(output / 'fixture-inputs.json', compile_fixtures)
save(output / 'compiled-inputs.json', compiled_inputs)
(output / 'runner.py').write_bytes(Path(__file__).read_bytes())
receipt = {'status': 'initializing', 'started_utc': now(), 'profile': profile, 'source_files': len(pins),
           'normal_input_kind': args.normal_kind,
           'additional_compile_fixtures': compile_fixtures, 'compiled_input_files': len(compiled_inputs),
           'normal_qualification': str(normal_path), 'normal_qualification_sha256': sha(normal_path),
           'G06_qualification': str(completion_path), 'G06_qualification_sha256': sha(completion_path),
           'source_manifest_sha256': sha(output / 'source-inputs.json'), 'features': [],
           'normal_mains_before': expected_mains, 'library_unit_tests': False,
           'full_current_TCP_regression_verified': False, 'whole_goal_complete': False,
           'output': str(output), 'results': {}, 'process_history': []}
active_history_scope=None
persist = lambda: save(output / 'receipt.json', receipt)
persist()
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
class OwnHandle:pass
self_handle=OwnHandle();self_handle._handle=k.GetCurrentProcess()
save(output/'runner-identity.json',dict(pid=os.getpid(),creation_utc=creation_time(self_handle).isoformat()))
print('CURRENT1047_DEFAULT116='+str(output),flush=True)
# Retain exact compiled inputs before any compilation or possible failure.
source_archive = output / 'compiled-source.zip'
with zipfile.ZipFile(source_archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, expected in compiled_inputs.items():
        assert sha(ROOT / name) == expected
        z.write(ROOT / name, name)
with zipfile.ZipFile(source_archive) as z:
    assert set(z.namelist()) == set(compiled_inputs)
    for name, expected in compiled_inputs.items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected
receipt['compiled_source_archive_sha256'] = sha(source_archive)
receipt['compiled_source_archive_readback_verified'] = True
persist()

env = os.environ.copy()
env.update({'RUSTC': str(TOOLS / 'rustc.exe'), 'RUSTDOC': str(TOOLS / 'rustdoc.exe'),
            'TEMP': str(PROJECT / 'target/current/scratch'), 'TMP': str(PROJECT / 'target/current/scratch')})
env['PATH'] = str(TOOLS) + os.pathsep + env['PATH']
env['CARGO_TARGET_DIR']=str(PROJECT/'target/current')
for name in list(env):
    if name.startswith('LITO_'):del env[name]

def execute(command, directory, child_env, timeout=None):
    directory.mkdir(parents=True, exist_ok=True)
    with (directory / 'stdout.log').open('wb') as out, (directory / 'stderr.log').open('wb') as err:
        child = subprocess.Popen(command, cwd=PROJECT, env=child_env, stdout=out, stderr=err,
                                 creationflags=subprocess.CREATE_NO_WINDOW)
        created = creation_time(child)
        receipt['owned_process'] = {'pid': child.pid, 'creation_utc': created.isoformat(), 'arguments': command}
        receipt['process_history'].append(receipt['owned_process'])
        persist()
        try:
            code = child.wait(timeout=timeout)
            timed_out = False
        except subprocess.TimeoutExpired:
            stop_owned(child, command, created)
            code, timed_out = child.returncode, True
        receipt['owned_process'].update(exit_code=code, timed_out=timed_out, finished_utc=now())
        receipt['owned_process'] = None
        persist()
    return code, timed_out

try:
    cargo = str(TOOLS / 'cargo.exe')
    receipt['status'] = 'reading_authoritative_target_inventory'; persist()
    code, _ = execute([cargo, 'metadata', '--no-deps', '--locked', '--offline', '--format-version', '1'], output / 'metadata', env)
    assert code == 0, 'Cargo metadata failed'
    metadata = read(output / 'metadata/stdout.log')
    package = next(p for p in metadata['packages'] if Path(p['manifest_path']).resolve() == PROJECT / 'Cargo.toml')
    tests = [t for t in package['targets'] if 'test' in t['kind']]
    suites = sorted(t['name'] for t in tests if not t.get('required-features'))
    excluded = {t['name']: t['required-features'] for t in tests if t.get('required-features')}
    assert len(suites) == 116 and {'native_environment','native_world_lighting','native_world_lighting_scene','native_light_lifecycle','native_background','native_background_geometry','native_background_pixels','native_object_dispatch','native_green_clock','native_startup','native_selector_only','native_ability_buffs','native_ability_world','native_additional_finite_resources','native_ability_live'} <= set(suites) and excluded == {'native_teleport_controller': ['teleport-wire-test']}
    assert all(re.fullmatch(r'[A-Za-z0-9_]+', n) for n in suites)
    receipt.update(suites=suites, required_feature_targets_not_selected=excluded, status='compiling_default_targets')
    command = [cargo, 'test', '--no-run', '--locked', '--offline', '-j1', '--target', 'x86_64-pc-windows-msvc',
               '--target-dir', 'target/current', '--message-format=json', '--bin', 'LiToModern']
    if profile == 'Release':
        command.append('--release')
    for name in suites:
        command += ['--test', name]
    receipt['build_arguments'] = command; persist()
    code, _ = execute(command, output / 'build', env)
    receipt['compile_exit_code'] = code; persist()
    assert code == 0, 'Default target compilation failed; preserve build logs'
    check_inputs()
    messages = [json.loads(x) for x in (output / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if x.startswith('{')]
    assert any(x.get('reason') == 'build-finished' and x['success'] for x in messages)
    artifacts = [x for x in messages if x.get('reason') == 'compiler-artifact' and x.get('executable') and x['profile']['test']]
    products = {}
    for artifact in artifacts:
        assert artifact['features'] == []
        name = 'main_unit' if artifact['target']['kind'] == ['bin'] and artifact['target']['name'] == 'LiToModern' else artifact['target']['name']
        assert name not in products
        path = Path(artifact['executable']).resolve(strict=True)
        assert path.is_relative_to(PROJECT / 'target/current')
        with path.open('rb') as image:
            header=image.read(64);assert header[:2]==b'MZ'
            image.seek(int.from_bytes(header[60:64],'little'));pe=image.read(26)
            assert pe[:4]==b'PE\0\0' and int.from_bytes(pe[4:6],'little')==0x8664 and int.from_bytes(pe[24:26],'little')==0x20b
        products[name] = {'path': str(path), 'sha256': sha(path)}
    assert set(products) == set(suites) | {'main_unit'}
    receipt['status'] = 'retaining_exact_test_products'; persist()
    archive = output / 'test-products.zip'
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        for name, product in products.items():
            z.write(product['path'], name + '.exe')
    with zipfile.ZipFile(archive) as z:
        for name, product in products.items():
            assert hashlib.sha256(z.read(name + '.exe')).hexdigest().upper() == product['sha256']
    save(output / 'product-retention.json', {'products': products, 'archive': str(archive), 'archive_sha256': sha(archive), 'readback_verified': True})
    fixture_path = STAGE / 'library-pet-fixture-map.json'
    fixtures = read(fixture_path)
    for name, expected in fixtures['files'].items():
        assert sha(ROOT / name) == expected
    save(output / 'pet-fixture-inputs.json', fixtures)
    for index, name in enumerate(['main_unit'] + suites):
        case = output / 'cases' / name
        case.mkdir(parents=True)
        product = products[name]
        assert sha(product['path']) == product['sha256']
        receipt.update(status='running_default_targets', active_suite=name, completed_suites=index); persist()
        case_env = env.copy()
        if name == 'native_library_regression':
            for key, value in fixtures['inputs'].items():
                case_env[key] = value
            for key, value in fixtures['outputs'].items():
                destination = case / value['name']
                if value['kind'] == 'directory':
                    destination.mkdir()
                case_env[key] = str(destination)
            code, _ = execute(['python', '-X', 'utf8', str(ROOT / '維護工具/preserve-legacy-library-reports.py'), 'snapshot', str(case)], case / 'history-before', env)
            assert code == 0
            active_history_scope=case
        code, timed_out = execute([product['path'], '--list'], case / 'inventory', case_env, timeout=60)
        assert code == 0 and not timed_out
        names = re.findall(r'^(.+): test$', (case / 'inventory/stdout.log').read_text(encoding='utf-8-sig'), re.M)
        code, timed_out = execute([product['path'], '--test-threads=1'], case / 'runtime', case_env, timeout=600)
        text = (case / 'runtime/stdout.log').read_text(encoding='utf-8-sig')
        matches = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', text)
        assert matches, ('missing terminal test result', name, code, timed_out)
        terminal, passed, failed, ignored, measured, filtered = matches[-1]
        passed, failed, ignored, filtered = map(int, [passed, failed, ignored, filtered])
        assert filtered == 0 and passed + failed + ignored == len(names)
        assert sha(product['path']) == product['sha256']
        if name == 'native_library_regression':
            restored, _ = execute(['python', '-X', 'utf8', str(ROOT / '維護工具/preserve-legacy-library-reports.py'), 'collect-restore', str(case)], case / 'history-after', env)
            assert restored == 0
            active_history_scope=None
        receipt['results'][name] = {'passed': passed, 'failed': failed, 'ignored': ignored, 'exit_code': code,
                                   'timed_out': timed_out, 'inventory_count': len(names), 'product_sha256': product['sha256'],
                                   'runtime_log_sha256': sha(case / 'runtime/stdout.log')}
        receipt['completed_suites'] = index + 1; persist()
        if (index + 1) % 10 == 0 or failed:
            print(f'{profile}: completed {index + 1}/{len(suites) + 1}; current {name} {passed} passed/{failed} failed/{ignored} ignored', flush=True)
    check_inputs()
    for name, expected in fixtures['files'].items():
        assert sha(ROOT / name) == expected
    receipt['normal_mains_after'] = mains()
    assert receipt['normal_mains_after'] == expected_mains, 'Normal main changed during default regression'
    receipt.update(sources_unchanged=True, fixture_inputs_unchanged=True, active_suite=None,
                   passed=sum(x['passed'] for x in receipt['results'].values()),
                   failed=sum(x['failed'] for x in receipt['results'].values()),
                   ignored=sum(x['ignored'] for x in receipt['results'].values()))
    assert len(receipt['results']) == len(suites) + 1
    assert all(x['exit_code'] == 0 and not x['timed_out'] for x in receipt['results'].values()) and receipt['failed'] == 0
    receipt['status'] = 'verified_current_default116_integration_targets_and_main_units'
except BaseException as error:
    if active_history_scope is not None:
        restored,_=execute(['python','-X','utf8',str(ROOT/'維護工具/preserve-legacy-library-reports.py'),'collect-restore',str(active_history_scope)],active_history_scope/'history-failure-recovery',env)
        receipt['failure_history_restoration_exit_code']=restored
    receipt.update(status='failed', error=f'{type(error).__name__}: {error}')
    raise
finally:
    receipt['finished_utc'] = now(); persist()
print(json.dumps({k: receipt[k] for k in ['status', 'profile', 'passed', 'failed', 'ignored', 'output']}, ensure_ascii=False), flush=True)
