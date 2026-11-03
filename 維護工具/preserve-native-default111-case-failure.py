"""Seal one terminal Rust failure while the original broad batch continues."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--target', required=True)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/default-native-regression')
assert re.fullmatch(r'[a-z][a-z0-9_]*', a.target)
c = read(run/'receipt.json')
result = c['results'][a.target]
assert result['exit_code'] != 0 and result['failed'] > 0 and not result['timed_out']
inputs = read(run/'compiled-inputs.json')
assert len(inputs) == 1025 and all(sha(ROOT/n) == h for n,h in inputs.items())
assert sha(run/'source-inputs.json') == c['source_manifest_sha256']
assert sha(run/'compiled-source.zip') == c['compiled_source_archive_sha256']
with zipfile.ZipFile(run/'compiled-source.zip') as z:
    assert len(z.namelist()) == len(inputs) and set(z.namelist()) == set(inputs)
    for name, expected in inputs.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == expected
messages = [json.loads(line) for line in (run/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
artifacts = [m for m in messages if m.get('reason') == 'compiler-artifact' and m.get('executable') and m['target']['name'] == a.target]
assert len(artifacts) == 1
artifact = artifacts[0]
product = Path(artifact['executable']).resolve(strict=True)
source = Path(artifact['target']['src_path']).resolve(strict=True)
assert product.is_relative_to(ROOT/'LiTo381-Modern/target/current') and source.is_relative_to(ROOT/'LiTo381-Modern/tests')
assert sha(product) == result['product_sha256']
retention = read(run/'product-retention.json')
assert retention['readback_verified'] and retention['products'][a.target]['sha256'] == sha(product)
with zipfile.ZipFile(retention['archive']) as z:
    with z.open(a.target+'.exe') as f:
        assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == sha(product)
jobs = [p for p in c['process_history'] if Path(p['arguments'][0]).resolve() == product]
assert len(jobs) == 2 and all('finished_utc' in p for p in jobs)
assert {p['exit_code'] for p in jobs} == {0, result['exit_code']}
ids = ','.join(str(p['pid']) for p in jobs)
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert observed.returncode == 0 and not observed.stderr
live = json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for row in live:
    for job in jobs:
        if row['ProcessId'] == job['pid']:
            assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds()) > .001
case = run/'cases'/a.target
assert sha(case/'runtime/stdout.log') == result['runtime_log_sha256']
out = run/('failure-case-'+a.target)
out.mkdir()
save(out/'receipt-at-capture.json', c)
save(out/'compiler-artifact.json', artifact)
(out/'CIM.json').write_bytes(observed.stdout)
(out/'preserver.py').write_bytes(Path(__file__).read_bytes())
files = {name:out/name for name in ['receipt-at-capture.json', 'compiler-artifact.json', 'CIM.json', 'preserver.py']}
files.update({'product.exe':product, 'test.rs':source})
for name in ['compiled-source.zip', 'compiled-inputs.json', 'source-inputs.json']:
    files[name] = run/name
for phase in ['inventory', 'runtime']:
    for name in ['stdout.log', 'stderr.log']:
        files[phase+'/'+name] = case/phase/name
hashes = {name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(out/'raw.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name,p in files.items():
        z.write(p, name, compress_type=zipfile.ZIP_STORED if p.suffix.lower() == '.zip' else zipfile.ZIP_DEFLATED)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert len(z.namelist()) == len(hashes) and set(z.namelist()) == set(hashes)
    for name, expected in hashes.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == expected
record = dict(status='retained_actual_terminal_broad_case_failure_before_candidate_repair',
    target=a.target, profile=c['profile'], source_manifest_sha256=c['source_manifest_sha256'],
    original_Rust_exit_code=result['exit_code'], result=result, recorded_case_processes=jobs,
    recorded_case_processes_all_terminal=True, original_broad_batch_status_at_capture=c['status'],
    whole_broad_batch_terminal_inferred=False, product_source_changed=False,
    raw_archive_sha256=sha(out/'raw.zip'), raw_files=hashes, whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
save(out/'readback.json', record)
print(json.dumps(dict(readback=str(out/'readback.json'), archive_sha256=record['raw_archive_sha256'],
    original_Rust_exit_code=result['exit_code'], whole_broad_batch_terminal_inferred=False), ensure_ascii=False))
