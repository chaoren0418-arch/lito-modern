"""Seal an actual related-validation failure before any source or harness repair."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current')
c=read(run/'cohort.json');assert c['status']=='failed_preserved' and c['owned_process'] is None
failed=[p for p in c['process_history'] if p['exit_code']!=0];assert failed and all('exit_code' in p for p in c['process_history'])
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1025
for n,h in inputs.items():assert sha(ROOT/n)==h,n
ids=','.join(str(p['pid']) for p in c['process_history'])
script="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.run(['pwsh.exe','-NoProfile','-Command',script],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert observed.returncode==0 and not observed.stderr
live=json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for p in c['process_history']:
        if row['ProcessId']==p['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(p['creation_utc'])).total_seconds())>0.001
files={'run/'+p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file()}
app=Path(c['application'])
for name in ['application.json','base-source-inputs.json','base-compiled-inputs.json','applied-source-inputs.json','before.zip','candidate.json','independent-red-readback.json']:
    if (app/name).exists():files['application/'+name]=app/name
out=run/'failure-before-repair';out.mkdir();(out/'preserver.py').write_bytes(Path(__file__).read_bytes());files['preserver.py']=out/'preserver.py'
(out/'process-observation.json').write_bytes(observed.stdout);files['process-observation.json']=out/'process-observation.json'
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(hashes)
    for n,h in hashes.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
record=dict(status='retained_related_validation_failure_before_repair',original_session_id=a.session_id,observed_terminal_exit_code=1,
    terminal_tool_chunk_id=a.terminal_chunk,source_manifest_sha256=c['source_manifest_sha256'],failed_profile=c['active_profile'],failed_processes=failed,
    profile_results={p:dict(tests=v['tests'],normal=v.get('normal')) for p,v in c['profiles'].items()},
    recorded_direct_children=len(c['process_history']),recorded_children_all_terminal=True,
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(readback=str(out/'readback.json'),archive_sha256=record['raw_archive_sha256'],actual_wrapper_exit_code=1),ensure_ascii=False))
