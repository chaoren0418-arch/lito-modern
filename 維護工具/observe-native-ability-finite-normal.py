"""Close the actual normal wrapper, both qualified profiles and recorded identities."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args();assert a.terminal_exit_code==0 and a.terminal_chunk.lower()not in['','pending','unknown']
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-ability-finite-normal')
c=read(run/'cohort.json');assert c['status']=='executed_ability_finite_normals_related' and c['owned_process'] is None
profiles={}
for name in ['Debug','Release']:
    p=next(run.glob(name+'-qualified-*/qualified.json'));q=read(p)
    assert q['status']=='independently_qualified_ability_finite_normal_related_scope' and q['passed']==31 and q['selected_library_passed']==0 and q['new_graphics_verified']==11 and len(q['targets'])==5 and q['recorded_owned_all_terminal']
    assert sha(q['normal']['path'])==q['normal']['sha256'] and q['source_manifest_sha256']==c['source_manifest_sha256']
    profiles[name]=dict(qualification=str(p),qualification_sha256=sha(p),normal_sha256=q['normal']['sha256'],passed=q['passed'])
for name,digest in read(run/'compiled-inputs.json').items():assert sha(ROOT/name)==digest
jobs=[read(run/'original-runner-process.json')]+c['process_history']
for job in c['observed_compilers']:jobs.append(dict(pid=job['ProcessId'],creation_utc=job['created'],compiler_only_terminal_observation=True))
unique={}
for job in jobs:
    if 'exit_code' in job:assert job['exit_code']==0
    unique[(job['pid'],datetime.fromisoformat(job['creation_utc'].replace('Z','+00:00')))]=job
ids=','.join(str(pid) for pid,born in unique)
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for (pid,born),job in unique.items():
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-born).total_seconds())>.001
record=dict(status='independently_qualified_ability_finite_normal_both_profiles_and_terminal_processes',source_manifest_sha256=c['source_manifest_sha256'],cohort_sha256=sha(run/'cohort.json'),profiles=profiles,
    original_session=a.session,observed_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk,recorded_original_owned_identities=len(unique),recorded_original_owned_processes_all_terminal=True,
    identities=list(unique.values()),qualification_scope='unique-entry normal products, scene23, ability7 and opted-in original resources11 per profile; live body unexecuted; no broad/library/fullGUI promotion',full_library_verified=False,full_G06_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(run/'completion-observation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(observation=str(run/'completion-observation.json'),profiles=2,passed_each={n:v['passed']for n,v in profiles.items()},terminal_identities=len(unique)),ensure_ascii=False))
