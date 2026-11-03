"""Observe recorded process identities after the actual continuation terminal."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,time
ROOT=Path(__file__).resolve().parents[1]
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int);ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-keyboard-focus-controller') and a.terminal_exit_code==0
c=read(run/'cohort.json');q=read(run/'qualified/qualified.json');assert q['original_session_id']==a.session_id and q['observed_terminal_exit_code']==0 and q['actual_formal_sessions']==96
assert c['owned_process'] is None and all(p['exit_code']==0 for p in c['process_history'])
identities=[]
for p in c['process_history']:identities.append((p['pid'],p['creation_utc']))
runner=read(run/'runner-identity.json');identities.append((runner['ProcessId'],runner['creation_utc']))
prior=read(Path(c['retained_initial_failed_cohort'])/'cohort.json') if c.get('retained_initial_failed_cohort') else None
if prior:
    for p in prior['process_history']:identities.append((p['pid'],p['creation_utc']))
for profile,info in c['profiles'].items():
    for scenario,binding in info['cases'].items():
        r=read(Path(binding['directory'])/'receipt.json')
        for p in r['owned_processes']:identities.append((p['pid'],p['start_utc']))
failed=Path(prior['profiles']['Debug']['cases']['refused']['directory']) if prior else None
if failed:
    for p in read(failed/'receipt.json')['owned_processes']:identities.append((p['pid'],p['start_utc']))
identities=sorted(set(identities));ids=','.join(str(pid) for pid in sorted({pid for pid,_ in identities}))
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
result=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert result.returncode==0 and not result.stderr
live=json.loads(result.stdout.decode('utf-8-sig')) if result.stdout.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for pid,created in identities:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(created)).total_seconds())>0.001
startup=read(c['startup_qualification']);receipt=read(Path(c['startup_prerequisite'])/'cohort.json')
for profile,info in receipt['profiles'].items():assert sha(info['normal']['path'])==c['normal_mains'][profile]==startup['profiles'][profile]['normal']['sha256']
pins=read(run/'source-inputs.json');assert len(pins)==1047
for name,digest in pins.items():assert sha(ROOT/name)==digest
out=run/'completion-observation.json';assert not out.exists();(run/'completion-CIM.json').write_bytes(result.stdout);(run/'completion-observer.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='independently_observed_terminal_current1047_G06_both_profiles_and_recorded_owned_processes',original_session_id=a.session_id,observed_terminal_exit_code=0,qualification=str(run/'qualified/qualified.json'),qualification_sha256=sha(run/'qualified/qualified.json'),source_manifest_sha256=c['source_manifest_sha256'],actual_qualified_formal_sessions=96,normal_mains=c['normal_mains'],recorded_original_owned_identities=len(identities),recorded_original_owned_processes_all_terminal=True,PID_reuse_candidates=live,retained_failed_attempt=str(failed) if failed else None,failed_attempt_not_relabelled=failed is not None,GUI_verified=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
out.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(completion=str(out),qualified_sessions=96,recorded_owned_identities=len(identities)),ensure_ascii=False))
