"""Read back the terminal timing diagnostic, with no functional qualification."""
from pathlib import Path
from datetime import datetime
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'驗證紀錄/current/native-G06-controller/20261017-034558-407944'
BUILD=BASE/'failure-timing-diagnostic-20261017-035645-444720'
CASE=ROOT/'驗證紀錄/current/native-teleport-failure/20261017-035852-425'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
b=read(BUILD/'build.json');r=read(CASE/'receipt.json');runtime=Path(r['runtime'])
assert b['build_exit_code']==0 and r['test_exit_code']==101 and r['server_exit_code']==1 and r['status']=='failed'
assert sha(r['immutable_test_binary'])==sha(b['executable'])==b['executable_sha256']==r['test_binary_sha256']
assert sha(BUILD/'probe.rs')==b['source_sha256'] and bytes([0]) not in (BUILD/'probe.rs').read_bytes()
assert sha(ROOT/'LiTo381-Modern/tests/native_teleport_failure.rs')==b['original_test_source_sha256']
for info in b['library_inputs'].values():assert sha(info['path'])==info['sha256']
timings={role:[json.loads(s) for s in (runtime/f'{role}-timing.jsonl').read_text(encoding='utf-8').splitlines()]
    if (runtime/f'{role}-timing.jsonl').exists() else [] for role in ['a','b']}
assert len(timings['a'])==7 and timings['b']==[]
assert read(runtime/'b-recovery-error.json')['error']=='timeout at BattlePending'
identities=[(p['pid'],p['start_utc']) for p in r['owned_processes']]
ids=','.join(str(pid) for pid,_ in identities)
script="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.run(['pwsh.exe','-NoProfile','-Command',script],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert observed.returncode==0 and not observed.stderr
live=json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    assert all(row['ProcessId']!=pid or abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(created)).total_seconds())>0.01 for pid,created in identities)
files={}
for folder,prefix in [(BUILD,'build/'),(CASE,'case/'),(runtime,'runtime/')]:
    for p in folder.rglob('*'):
        if p.is_file() and p.suffix.lower()!='.pdb':
            assert p.resolve().is_relative_to(folder.resolve()) and not p.is_symlink()
            files[prefix+p.relative_to(folder).as_posix()]=p
files['host-load-observation.json']=BASE/'host-load-observation-20261017-0357.json'
out=BUILD/'runtime-failure-readback';out.mkdir()
(out/'independent-reader.py').write_bytes(Path(__file__).read_bytes());files['independent-reader.py']=out/'independent-reader.py'
(out/'process-observation.json').write_bytes(observed.stdout);files['process-observation.json']=out/'process-observation.json'
pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(pins)
    for n,h in pins.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
record=dict(status='retained_failed_same_library_recovery_phase_timing_diagnostic',
    original_session_id=75086,observed_terminal_exit_code=1,terminal_tool_chunk_id='f388db',
    source_manifest_sha256=b['source_manifest_sha256'],test_exit_code=101,server_exit_code=1,
    phase_timings=timings,case=str(CASE),recorded_owned_processes_all_terminal=True,
    actual_qualified_formal_sessions=0,product_sources_changed=False,deadlines_unchanged=True,
    observed_failure='B waits before its initial Model load and times out; A completes recovery at 85.8 seconds, then sees EOF after a fixture accept timeout.',
    original_eight_second_failure_root_cause_proven=False,other_host_load_not_excluded=True,
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,performance_accepted=False,GUI_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(readback=str(out/'readback.json'),archive_sha256=record['raw_archive_sha256']),ensure_ascii=False))
