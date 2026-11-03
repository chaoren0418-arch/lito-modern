"""Seal a completed broad-test failure before changing canonical sources."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

def archive(path, digest, files):
    assert sha(path) == digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist()) == len(files) and set(z.namelist()) == set(files)
        for name, expected in files.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == expected

ap = argparse.ArgumentParser()
ap.add_argument('cohort', type=Path)
ap.add_argument('--session-id', required=True, type=int)
ap.add_argument('--terminal-exit-code', required=True, type=int)
ap.add_argument('--terminal-chunk', required=True)
a = ap.parse_args()
run = a.cohort.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-default110')
c = read(run/'cohort.json')
assert c['status'] == 'failed_preserved' and c['owned_process'] is None and a.terminal_exit_code != 0
binding = read(run/'original-session-binding.json')
assert binding['original_session_id'] == a.session_id and binding['pipeline_pid'] == c['pipeline_pid']
assert all(p['exit_code'] is not None for p in c['process_history'])
identities = [(c['pipeline_pid'], c['pipeline_creation_utc'])]+[(p['pid'], p['creation_utc']) for p in c['process_history']]
files = {'pipeline/'+p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file()}
profiles, all_failures = {}, {}
for profile, info in c['profiles'].items():
    inner = Path(info['run']).resolve(strict=True)
    assert inner.is_relative_to(ROOT/'驗證紀錄/current/default-native-regression')
    receipt = read(inner/'receipt.json')
    assert receipt['owned_process'] is None and receipt['compile_exit_code'] == 0
    assert len(receipt['results']) == receipt['completed_suites'] == 111
    assert all('finished_utc' in p for p in receipt['process_history'])
    assert receipt['source_manifest_sha256'] == c['source_manifest_sha256']
    inputs = read(inner/'compiled-inputs.json')
    assert len(inputs) == 1025 and all(sha(ROOT/name) == h for name,h in inputs.items())
    assert sha(inner/'source-inputs.json') == c['source_manifest_sha256']
    archive(inner/'compiled-source.zip', receipt['compiled_source_archive_sha256'], inputs)
    retained = read(inner/'product-retention.json')
    assert retained['readback_verified'] and len(retained['products']) == 111
    archive(retained['archive'], retained['archive_sha256'], {name+'.exe':v['sha256'] for name,v in retained['products'].items()})
    for name, product in retained['products'].items():
        assert sha(product['path']) == product['sha256'] == receipt['results'][name]['product_sha256']
    pet = read(inner/'pet-fixture-inputs.json')
    assert len(pet['files']) == 353 and all(sha(ROOT/name) == h for name,h in pet['files'].items())
    legacy = inner/'cases/native_library_regression'
    before = read(legacy/'legacy-reports-before.json')
    restored = read(legacy/'legacy-report-restoration.json')
    assert before['readback_verified'] and restored['existing_reports_restored']
    assert before['archive_sha256'] == restored['before_archive_sha256']
    assert all(sha(ROOT/'LiTo381-Modern/logs/continuation-20261003'/name) == h for name,h in before['files'].items())
    failed = {name:v for name,v in receipt['results'].items() if v['exit_code'] != 0 or v['failed'] or v['timed_out']}
    profiles[profile] = dict(run=str(inner), actual_wrapper_exit_code=info['actual_runner_exit_code'],
        results=receipt['results'], failed_cases=failed, historical_reports_restored=len(before['files']), historical_pet_files=353)
    all_failures.update({profile+'/'+name:v for name,v in failed.items()})
    identities += [(p['pid'], p['creation_utc']) for p in receipt['process_history']]
    for p in inner.rglob('*'):
        if p.is_file():
            assert p.resolve().is_relative_to(ROOT)
            files['profiles/'+profile+'/'+p.relative_to(inner).as_posix()] = p
assert all_failures
for name,digest in c['validation_pins'].items():
    assert sha(run/name) == sha(ROOT/'維護工具'/name) == digest
for key in ['normal_qualification', 'G06_qualification']:
    p = Path(c[key]).resolve(strict=True)
    assert p.is_relative_to(ROOT) and sha(p) == c[key+'_sha256']
    files['upstream/'+key+'.json'] = p
identities = sorted(set(identities))
ids = ','.join(str(pid) for pid in sorted({pid for pid,_ in identities}))
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert observed.returncode == 0 and not observed.stderr
live = json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for row in live:
    for pid, created in identities:
        if row['ProcessId'] == pid:
            assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(created)).total_seconds()) > .001
out = run/'failure-before-repair'
out.mkdir()
(out/'CIM.json').write_bytes(observed.stdout)
(out/'preserver.py').write_bytes(Path(__file__).read_bytes())
files.update({'CIM.json':out/'CIM.json', 'preserver.py':out/'preserver.py'})
hashes = {name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(out/'raw.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name,p in files.items():
        z.write(p, name, compress_type=zipfile.ZIP_STORED if p.suffix.lower() == '.zip' else zipfile.ZIP_DEFLATED)
archive(out/'raw.zip', sha(out/'raw.zip'), hashes)
record = dict(status='retained_actual_default111_terminal_failure_before_source_repair',
    original_session_id=a.session_id, observed_terminal_exit_code=a.terminal_exit_code, terminal_tool_chunk_id=a.terminal_chunk,
    source_manifest_sha256=c['source_manifest_sha256'], failed_profile=c['active_profile'], profiles=profiles,
    failed_cases=all_failures, recorded_owned_identities=len(identities), recorded_owned_all_terminal=True,
    PID_reuse_candidates=live, raw_archive_sha256=sha(out/'raw.zip'), raw_files=hashes,
    whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
save(out/'readback.json', record)
print(json.dumps(dict(readback=str(out/'readback.json'), archive_sha256=record['raw_archive_sha256'],
    original_exit_code=a.terminal_exit_code, failed_cases=list(all_failures), recorded_owned_identities=len(identities)), ensure_ascii=False))
