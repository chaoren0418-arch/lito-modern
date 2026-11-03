"""Bind completed related profiles without inventing the wrapper's exit code."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def archive(path, digest, files):
    assert sha(path) == digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist()) == len(files) and set(z.namelist()) == set(files)
        for name, digest in files.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-retained-geometry')
c = read(run / 'cohort.json')
assert c['status'] == 'executed_retained_geometry_related_tests_and_unique_normals'
assert c['owned_process'] is None and len(c['process_history']) == 26
assert all(p['exit_code'] == 0 for p in c['process_history'])
inputs = read(run / 'compiled-inputs.json')
assert len(inputs) == 1025
assert len(read(run / 'source-inputs.json')) == 1024
assert sha(run / 'source-inputs.json') == c['source_manifest_sha256']
assert all(sha(ROOT / name) == digest for name, digest in inputs.items())
assert sha(ROOT / '啟動現代客戶端.ps1') == c['entry_sha256']
source = c['compiled_source_archive']
archive(source['path'], source['sha256'], inputs)
profiles = {}
timings = {}
for profile in ['Debug', 'Release']:
    path = run / profile / 'profile-qualified/qualified.json'
    q = read(path)
    assert q['status'] == 'independently_qualified_retained_geometry_related_profile'
    assert q['source_manifest_sha256'] == c['source_manifest_sha256']
    assert q['counts'] == dict(passed=52 if profile == 'Debug' else 48, failed=0, ignored=3)
    archive(path.parent / 'raw.zip', q['raw_archive_sha256'], q['raw_files'])
    info = c['profiles'][profile]
    product = info['test_products_archive']
    archive(product['path'], product['sha256'], {name+'.exe':p['sha256'] for name,p in info['products'].items()})
    for p in info['products'].values():
        assert sha(p['path']) == p['sha256']
    normal = info['normal']
    assert sha(normal['path']) == q['normal_sha256'] == normal['sha256']
    archive(normal['archive']['path'], normal['archive']['sha256'], {'LiToModern.exe':normal['sha256']})
    terrain = read(run / profile / 'terrain-output/563-static.json')
    timings[profile] = {k:terrain[k]['load_profile_us'] for k in ['first','before_boundary','after_boundary']}
    profiles[profile] = dict(qualification=str(path), qualification_sha256=sha(path),
        counts=q['counts'], normal_sha256=q['normal_sha256'], actual_test_executables=10)

ids = ','.join(str(p['pid']) for p in c['process_history'])
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p = subprocess.run(['powershell.exe','-NoProfile','-Command',command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode == 0 and not p.stderr
live = json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live,dict): live = [live]
for row in live:
    for job in c['process_history']:
        if row['ProcessId'] == job['pid']:
            assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds()) > .001

observation = read(run / 'original-session-observation.json')
assert observation['original_session_id'] == 4754 and observation['handle_closed']
assert observation['terminal_exit_code'] == 0 and observation['terminal_tool_chunk_id']
files = {'observer.py':Path(__file__), 'cohort.json':run/'cohort.json',
    'original-session-observation.json':run/'original-session-observation.json'}
hashes = {name:sha(p) for name,p in files.items()}
raw = run / 'completion-observation-raw.zip'
with zipfile.ZipFile(raw,'x',zipfile.ZIP_DEFLATED) as z:
    for name,p in files.items(): z.write(p,name)
archive(raw,sha(raw),hashes)
q = dict(status='independently_qualified_retained_geometry_both_profiles',
    original_session_id=4754, observed_terminal_exit_code=0,
    wrapper_exit_code_unavailable=False, wrapper_handle_closed=True,
    completion_basis='Both immutable profile qualifications and all twenty-six recorded direct child processes returned actual exit0; original tool session4754 returned actual terminal exit0.',
    terminal_tool_chunk_id=observation['terminal_tool_chunk_id'],
    source_files=1024, compiled_input_files=1025, source_manifest_sha256=c['source_manifest_sha256'],
    entry_sha256=c['entry_sha256'], profiles=profiles, recorded_owned_identities=26,
    recorded_owned_processes_all_terminal=True, load_profiles_us=timings,
    timing_scope='archive_pins_us on reuse measures metadata selection only; source guard checks execute within static_assets_us. total_us is the whole load. Synthetic authority fixture, not a live TCP performance qualification.',
    raw_archive_sha256=sha(raw), raw_files=hashes, actual_formal_sessions=0,
    full_current_regression_verified=False, new_siege_case_verified=False,
    GUI_verified=False, whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
(run/'completion-observation.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({k:q[k] for k in ['status','observed_terminal_exit_code','recorded_owned_identities','profiles']},ensure_ascii=False))
