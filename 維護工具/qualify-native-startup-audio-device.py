"""Bind actual WinMM lifecycle evidence to the current compiled x64 image."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, struct, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', type=int, required=True)
ap.add_argument('--terminal-exit-code', type=int, required=True)
a = ap.parse_args()
assert a.terminal_exit_code == 0
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-startup-audio-device')
c = read(run / 'run.json')
assert c['status'] == 'executed_retained_current_Debug_actual_WinMM_and_independent_comparison'
assert c['helper_exit_code'] == c['comparison_exit_code'] == 0 and c['profile'] == 'Debug'
build = Path(c['original_build'])
qpath = Path(c['original_qualification'])
assert sha(qpath) == c['original_qualification_sha256']
q = read(qpath)
assert q['status'] == 'independently_qualified_current_default110_integrations_and_main'
assert q['source_manifest_sha256'] == c['source_manifest_sha256'] and q['counts']['failed'] == 0
inputs = read(build / 'compiled-inputs.json')
assert len(inputs) == c['compiled_input_files'] == 1025
for name, h in inputs.items():
    assert sha(ROOT / name) == h
assert sha(build / 'compiled-source.zip') == c['compiled_source_archive_sha256'] == q['compiled_source_archive_sha256']
with zipfile.ZipFile(build / 'compiled-source.zip') as z:
    assert len(z.namelist()) == len(inputs) and set(z.namelist()) == set(inputs)
    for name, h in inputs.items():
        with z.open(name) as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == h
product = c['test_product']
assert sha(product['path']) == product['sha256']
assert sha(c['test_products_archive']) == c['test_products_archive_sha256'] == q['product_archive_sha256']
with zipfile.ZipFile(c['test_products_archive']) as z:
    with z.open('native_audio_device.exe') as stream:
        assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == product['sha256']
messages = [json.loads(line) for line in (build / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
artifacts = [m for m in messages if m.get('reason') == 'compiler-artifact' and m.get('executable')
    and Path(m['executable']).resolve() == Path(product['path']).resolve()]
assert len(artifacts) == 1 and artifacts[0]['features'] == [] and artifacts[0]['profile']['test']
assert artifacts[0]['target']['name'] == 'native_audio_device'
with Path(product['path']).open('rb') as f:
    assert f.read(2) == b'MZ'
    f.seek(0x3c)
    offset = struct.unpack('<I', f.read(4))[0]
    f.seek(offset)
    assert f.read(4) == b'PE\0\0' and struct.unpack('<H', f.read(2))[0] == 0x8664
    f.seek(offset + 24)
    assert struct.unpack('<H', f.read(2))[0] == 0x20b
for name, key in [('run-native-audio-device.ps1', 'helper_sha256'), ('verify-native-audio-device.py', 'verifier_sha256')]:
    assert sha(run / name) == sha(ROOT / '維護工具' / name) == c[key]
case = Path(c['case']).resolve(strict=True)
assert case.is_relative_to(ROOT / '驗證紀錄/current/native-audio-device')
r = read(case / 'receipt.json')
report = read(case / 'actual-audio-device.json')
comparison = read(case / 'audio-device-comparison.json')
assert r['status'] == 'verified_actual_native_audio_device_lifecycle' and r['test_exit_code'] == 0
assert r['build_profile'] == 'Debug' and sha(r['immutable_test_binary']) == r['test_binary_sha256'] == product['sha256']
assert comparison['receipt_sha256'] == sha(case / 'receipt.json')
assert comparison['report_sha256'] == sha(case / 'actual-audio-device.json')
assert comparison['validator_sha256'] == c['verifier_sha256'] and report == r['report']
assert [v['sound_id'] for v in report['natural_cases']] == [97, 99, 102]
assert report['handles_before_warmed_cycles'] == report['handles_after_cycles']
assert comparison['reset_cycles'] == 20 and comparison['simultaneous_voices'] == 4
assert not comparison['physical_audibility_verified'] and not comparison['native_GUI_verified']
for name, h in r['source_pins'].items():
    assert sha(case / 'source-snapshot' / name) == sha(ROOT / name) == h
for name, h in r['readonly_resource_pins'].items():
    assert sha(name) == h
identities = [(c['owned_helper']['pid'], c['owned_helper']['creation_utc']),
    (r['owned_process']['pid'], r['owned_process']['start_utc'])]
query = ' OR '.join('ProcessId=' + str(pid) for pid, _ in identities)
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); @(Get-CimInstance Win32_Process -Filter '" + query + "'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode == 0 and not p.stderr
live = json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for row in live:
    for pid, created in identities:
        if row['ProcessId'] == pid:
            assert abs((datetime.fromisoformat(row['created']) - datetime.fromisoformat(created)).total_seconds()) > 0.001
out = run / 'qualified'
out.mkdir()
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
(out / 'owned-CIM.json').write_bytes(p.stdout)
files = {'independent-qualifier.py': out / 'independent-qualifier.py', 'owned-CIM.json': out / 'owned-CIM.json'}
for folder, prefix in [(run, 'run/'), (case, 'case/')]:
    for f in folder.rglob('*'):
        if f.is_file() and out not in f.parents and f.suffix.lower() not in ['.exe', '.zip']:
            assert f.resolve().is_relative_to(folder) and not f.is_symlink()
            files[prefix + f.relative_to(folder).as_posix()] = f
hashes = {name: sha(p) for name, p in files.items()}
raw = out / 'raw.zip'
with zipfile.ZipFile(raw, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, p in files.items():
        z.write(p, name)
with zipfile.ZipFile(raw) as z:
    assert len(z.namelist()) == len(hashes) and set(z.namelist()) == set(hashes)
    for name, h in hashes.items():
        with z.open(name) as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == h
result = dict(status='independently_qualified_current_Debug_actual_WinMM_lifecycle',
    original_session_id=a.session_id, observed_terminal_exit_code=0, profile='Debug',
    source_files=1024, compiled_input_files=1025, source_manifest_sha256=c['source_manifest_sha256'],
    actual_test_sha256=product['sha256'], actual_test_executables=1, passed=1, failed=0,
    original_wav_ids=[97, 99, 102], natural_completion_cases=3, reset_cycles=20,
    simultaneous_voices=4, handles_before=report['handles_before_warmed_cycles'], handles_after=report['handles_after_cycles'],
    actual_opened_device_ids=sorted({v['opened_device_id'] for v in report['natural_cases']}),
    WinMM_mapper_is_not_physical_endpoint_identity=True, recorded_owned_identities=len(identities),
    recorded_owned_processes_terminal=True, compiled_source_archive_sha256=c['compiled_source_archive_sha256'],
    test_products_archive_sha256=c['test_products_archive_sha256'], raw_archive_sha256=sha(raw), raw_files=hashes,
    normal_EXE_runtime_verified=False, physical_audibility_verified=False, GUI_verified=False,
    unplug_or_no_device_verified=False, whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'qualified.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: result[k] for k in ['status', 'passed', 'failed', 'handles_before', 'handles_after']}, ensure_ascii=False))
