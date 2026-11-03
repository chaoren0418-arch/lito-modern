"""Retain a terminal G06 batch failure before changing any fixture or probe."""
from pathlib import Path
import argparse, hashlib, json, zipfile, subprocess
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int)
ap.add_argument('--terminal-exit-code',required=True,type=int);ap.add_argument('--terminal-chunk',required=True)
a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-G06-controller') and a.terminal_exit_code==1
c=read(run/'cohort.json');assert c['status']=='failed_preserved' and c['owned_process'] is None
profile=c['active_profile'];scenario=c['active_scenario'];binding=c['profiles'][profile]['cases'][scenario]
assert binding['exit_code']!=0
case=Path(binding['directory']);r=read(case/'receipt.json');runtime=Path(r['runtime'])
assert r['status']=='failed' and runtime.resolve().is_relative_to(ROOT/'LiTo381-Modern/target/current')
assert r['test_exit_code']==101
identities=[(p['pid'],p['creation_utc']) for p in c['process_history']]
identities += [(p['pid'],p['start_utc']) for p in r['owned_processes']]
ids=','.join(str(p) for p in sorted({p for p,_ in identities}))
script="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.run(['pwsh.exe','-NoProfile','-Command',script],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert observed.returncode==0 and not observed.stderr
live=json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
if isinstance(live,dict):live=[live]
from datetime import datetime
for p in live:
    assert all(p['ProcessId']!=pid or abs((datetime.fromisoformat(p['created'])-datetime.fromisoformat(created)).total_seconds())>0.01 for pid,created in identities)
files={}
for folder,prefix in [(run,'batch/'),(case,'failed-case/'),(runtime,'failed-runtime/')]:
    for p in folder.rglob('*'):
        if p.is_file():
            assert p.resolve().is_relative_to(folder.resolve()) and not p.is_symlink()
            files[prefix+p.relative_to(folder).as_posix()]=p
out=run/'failure-before-repair';out.mkdir()
(out/'preserver.py').write_bytes(Path(__file__).read_bytes());files['preserver.py']=out/'preserver.py'
(out/'process-observation.json').write_bytes(observed.stdout);files['process-observation.json']=out/'process-observation.json'
pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(pins)
    for n,h in pins.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
record=dict(status='retained_failed_G06_batch_before_repair',original_session_id=a.session_id,
    observed_terminal_exit_code=a.terminal_exit_code,terminal_tool_chunk_id=a.terminal_chunk,
    source_manifest_sha256=c['source_manifest_sha256'],failed_profile=profile,failed_scenario=scenario,
    test_exit_code=r['test_exit_code'],server_exit_code=r.get('server_exit_code'),case=str(case),
    recorded_process_identities=len(set(identities)),recorded_processes_all_terminal=True,
    retained_passed_cases={p:[s for s,v in info['cases'].items() if v['exit_code']==0] for p,info in c['profiles'].items()},
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,performance_accepted=False,GUI_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(readback=str(out/'readback.json'),archive_sha256=record['raw_archive_sha256'],failed_scenario=scenario),ensure_ascii=False))
