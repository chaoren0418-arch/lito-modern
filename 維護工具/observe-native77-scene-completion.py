"""Independently close the owned normal/related cohort after actual tool exit."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--debug',type=Path,required=True);ap.add_argument('--release',type=Path,required=True);ap.add_argument('--terminal-tool-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-scene-integration')
c=read(run/'cohort.json');assert c['status']=='executed_current_native77_normals_and_related_scopes' and c['owned_process'] is None and c['active_profile'] is None
assert len(c['process_history'])==96 and all(j['exit_code']==0 for j in c['process_history'])
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1031 and sha(run/'source-inputs.json')==c['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h
profiles={}
for profile,path in [('Debug',a.debug),('Release',a.release)]:
    path=path.resolve(strict=True);assert path.is_relative_to(run);q=read(path)
    assert q['status']=='independently_qualified_current_native77_normal_related_profile' and q['profile']==profile and q['source_manifest_sha256']==c['source_manifest_sha256']
    assert q['registered_total']==2426 and q['original_ignored_preserved']==473 and q['added_explicit_resource_optins']==10
    for kind in ['normal','test']:assert sha(q[kind]['path'])==q[kind]['sha256']==c['profiles'][profile][kind]['sha256']
    assert q['recorded_owned_all_terminal'] and len(q['recorded_jobs'])==48 and q['explicit_actual_asset_passed']==10
    profiles[profile]=dict(qualification=str(path),qualification_sha256=sha(path),normal_sha256=q['normal']['sha256'],test_sha256=q['test']['sha256'],
        related_library_passed=q['passed_related_bodies'],related_library_ignored=q['ignored_related_bodies'],explicit_actual_asset_passed=10,external_integration_passed=q['external_integration_passed'],external_integration_ignored=q['external_integration_ignored'])
driver=read(run/'original-runner-process.json')
identities=[dict(pid=driver['pid'],created=driver['creation_utc'],role='driver')]
identities.extend(dict(pid=j['pid'],created=j['creation_utc'],role=j['label']) for j in c['process_history'])
identities.extend(dict(pid=j['ProcessId'],created=j['created'],role='observed rustc') for j in c['observed_compilers'])
unique={}
for item in identities:unique[(item['pid'],datetime.fromisoformat(item['created'].replace('Z','+00:00')).astimezone(timezone.utc))]=item
ids=','.join(str(n) for n in sorted({item['pid'] for item in unique.values()}))
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for (pid,created),item in unique.items():
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-created).total_seconds())>.001,item
terminal=dict(original_session_id=38394,exit_code=0,terminal_tool_chunk=a.terminal_tool_chunk);save(run/'actual-tool-terminal-readback.json',terminal)
out=run/('completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'observer.py').write_bytes(Path(__file__).read_bytes());save(out/'cohort.json',c);save(out/'terminal-process-readback.json',dict(identities=list(unique.values()),CIM_rows=live,all_recorded_identities_terminal=True))
q=dict(status='independently_observed_current_native77_normals_and_related_both_profiles',run=str(run),original_session_id=38394,observed_terminal_exit_code=0,original_terminal_tool_chunk=a.terminal_tool_chunk,
    source_manifest_sha256=c['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,profiles=profiles,recorded_job_count=96,recorded_owned_identities=len(unique),recorded_owned_all_terminal=True,
    completed_cohort=str(out/'cohort.json'),completed_cohort_sha256=sha(out/'cohort.json'),process_readback=str(out/'terminal-process-readback.json'),process_readback_sha256=sha(out/'terminal-process-readback.json'),
    qualification_scope='current_normal_builds_native77_scene_related_library_and_12_external_targets_only',default_library_registered=2426,default_library_ignored=483,original_default_ignored_preserved=473,explicit_actual_asset_bodies_per_profile=10,
    full_library_bodies_executed=False,full_G06_verified=False,new_sessions=0,new_casts=0,real_audio_device_verified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'completion.json',q);print(json.dumps(dict(completion=str(out/'completion.json'),sha256=sha(out/'completion.json'),profiles=profiles,recorded_owned_identities=len(unique),whole_goal_complete=False),ensure_ascii=False))
