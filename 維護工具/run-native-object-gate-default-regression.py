"""Build actual object-gate108 default integration targets plus main; library remains separate."""
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

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
STAGE = ROOT / '驗證紀錄/current/native-teleport-wire'
TOOLS = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
now = lambda: datetime.now(timezone.utc).isoformat()

def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest().upper()

def save(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

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
    assert abs((datetime.fromisoformat(actual['created']) - created).total_seconds()) < 1
    assert actual['command'].strip() == subprocess.list2cmdline(args)
    process.kill()  # Popen retains the handle of this exact verified process.
    process.wait(timeout=10)

ap = argparse.ArgumentParser()
ap.add_argument('--profile', choices=['Debug', 'Release'], default='Debug')
ap.add_argument('--normal-qualification', type=Path, required=True)
args = ap.parse_args()
profile = args.profile
normal_path = args.normal_qualification.resolve(strict=True)
assert normal_path.is_relative_to(ROOT / '驗證紀錄/current')
normal = read(normal_path)
pins = read(normal_path.parent / 'source-inputs.json')
assert len(pins) == normal['source_files'] == 1019 and normal['features'] == []
# All compile fixtures belong to the complete1019 input manifest.
compile_fixtures = {}
compiled_inputs = pins | compile_fixtures

# This runner belongs to the next default scope, not the frozen GUI harness.
# No build, metadata query or test may begin until that exact GUI run is terminal.
assert normal['status']=='qualified_unique_entry_object_gate_normal_builds'
assert normal['source_manifest_sha256']=='C065A70D35F4E2BE5038019361F42F5997B306F41C5430511B39715B7C40E432'
completion=read(normal_path.parent/'GUI-completion-observation.json')
assert completion['status']=='independently_readback_verified_current1019_TW14_GUI_and_terminal_process'
assert completion['source_files']==1019 and completion['terminal_exit_code']==0
assert completion['normal_mains']=={p:v['sha256'] for p,v in normal['profiles'].items()}
root_literal=str(ROOT).replace("'","''")
idle_command="""[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot='ROOT_LITERAL'
@(Get-CimInstance Win32_Process|Where-Object {
 $_.Name -in @('cargo.exe','rustc.exe') -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($taskRoot)) -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -and
 ($_.Name -match '^(LiToModern|lito_modern-|native_|qualified-client|actual-client)' -or
 ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))
}|Select-Object ProcessId,Name,ExecutablePath,CommandLine)|ConvertTo-Json -Depth 4 -Compress
""".replace('ROOT_LITERAL',root_literal)
idle=subprocess.run(['powershell.exe','-NoProfile','-Command',idle_command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert idle.returncode==0 and not idle.stderr
assert not idle.stdout.strip() or json.loads(idle.stdout.decode('utf-8-sig'))==[], 'Live owned workload: wait on its original handle.'

def check_inputs():
    for name, expected in compiled_inputs.items():
        assert sha(ROOT / name) == expected, ('source drift', name)

def mains():
    return {p: sha(Path(v['path'])) for p, v in normal['profiles'].items()}

check_inputs()
expected_mains = {p: v['sha256'] for p, v in normal['profiles'].items()}
assert mains() == expected_mains
output = ROOT / '驗證紀錄/current/default-native-regression' / (datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')[:-3] + '-' + profile)
output.mkdir(parents=True, exist_ok=False)
save(output / 'source-inputs.json', pins)
save(output / 'fixture-inputs.json', compile_fixtures)
save(output / 'compiled-inputs.json', compiled_inputs)
(output / 'runner.py').write_bytes(Path(__file__).read_bytes())
receipt = {'status': 'initializing', 'started_utc': now(), 'profile': profile, 'source_files': len(pins),
           'additional_compile_fixtures': compile_fixtures, 'compiled_input_files': len(compiled_inputs),
           'normal_qualification': str(normal_path), 'normal_qualification_sha256': sha(normal_path),
           'source_manifest_sha256': sha(output / 'source-inputs.json'), 'features': [],
           'normal_mains_before': expected_mains, 'library_unit_tests': False,
           'full_current_TCP_regression_verified': False, 'whole_goal_complete': False,
           'output': str(output), 'results': {}}
persist = lambda: save(output / 'receipt.json', receipt)
persist()
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

def execute(command, directory, child_env, timeout=None):
    directory.mkdir(parents=True, exist_ok=True)
    with (directory / 'stdout.log').open('wb') as out, (directory / 'stderr.log').open('wb') as err:
        child = subprocess.Popen(command, cwd=PROJECT, env=child_env, stdout=out, stderr=err,
                                 creationflags=subprocess.CREATE_NO_WINDOW)
        created = creation_time(child)
        receipt['owned_process'] = {'pid': child.pid, 'creation_utc': created.isoformat(), 'arguments': command}
        persist()
        try:
            code = child.wait(timeout=timeout)
            timed_out = False
        except subprocess.TimeoutExpired:
            stop_owned(child, command, created)
            code, timed_out = child.returncode, True
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
    assert len(suites) == 108 and {'native_environment','native_world_lighting','native_world_lighting_scene','native_light_lifecycle','native_background','native_background_geometry','native_background_pixels','native_object_dispatch'} <= set(suites) and excluded == {'native_teleport_controller': ['teleport-wire-test']}
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
    receipt['status'] = 'verified_current_default108_integration_targets_and_main_units'
except BaseException as error:
    receipt.update(status='failed', error=f'{type(error).__name__}: {error}')
    raise
finally:
    receipt['finished_utc'] = now(); persist()
print(json.dumps({k: receipt[k] for k in ['status', 'profile', 'passed', 'failed', 'ignored', 'output']}, ensure_ascii=False), flush=True)
