"""Seal the actual terminal current-source related run; preserve full-goal pending scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);ap.add_argument('--tool-chunk-id',required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-selector-integration') and a.terminal_exit_code==0
c=read(run/'cohort.json');assert c['status']=='executed_current_selector_related_tests_and_unique_normals' and c['owned_process'] is None and all(v['exit_code']==0 for v in c['process_history']) and len(c['process_history'])==64
assert set(c['profiles'])=={'Debug','Release'};profiles={};files={'observer.py':Path(__file__),'cohort.json':run/'cohort.json','source-inputs.json':run/'source-inputs.json','compiled-inputs.json':run/'compiled-inputs.json','source-descriptor-equivalence.json':run/'source-descriptor-equivalence.json'}
for profile in ['Debug','Release']:
 p=run/profile/'profile-qualified/qualified.json';q=read(p);assert q['status']=='independently_qualified_current_normal_selector_related_profile' and q['profile']==profile and q['counts']['failed']==0 and q['new_registered_selector_S39_tests_passed']==4
 assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['actual_test_executables']==29 and q['recorded_profile_identities_terminal'] and q['recorded_profile_top_level_all_exit_zero'] and q['unique_entry_build_verified']
 assert sha(c['profiles'][profile]['normal']['path'])==q['normal_sha256'];archive=run/profile/'profile-qualified/raw.zip';assert sha(archive)==q['raw_archive_sha256']
 with zipfile.ZipFile(archive) as z:
  assert set(z.namelist())==set(q['raw_files'])
  for n,h in q['raw_files'].items():
   with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
 profiles[profile]=dict(qualification=str(p),qualification_sha256=sha(p),normal_sha256=q['normal_sha256'],counts=q['counts'],actual_test_executables=29,new_registered_selector_S39_tests_passed=4);files[profile+'-qualified.json']=p
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1029
for n,h in inputs.items():assert sha(ROOT/n)==h
ids=[(v['pid'],v['creation_utc']) for v in c['process_history']]+[(v['ProcessId'],v['CreationUtc']) for v in c['observed_release_compilers']]
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+','.join(str(pid) for pid,_ in ids)+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
 for pid,created in ids:
  if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created.replace('Z','+00:00'))).total_seconds())>.001
assert not(run/'completion-observation.json').exists();(run/'completion-owned-CIM.json').write_bytes(p.stdout);files['owned-CIM.json']=run/'completion-owned-CIM.json';hashes={n:sha(p) for n,p in files.items()}
raw=run/'completion-observation-raw.zip'
with zipfile.ZipFile(raw,'x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
q=dict(status='independently_qualified_current_normal_selector_both_profiles',original_session_id=a.session_id,observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,source_files=1028,compiled_input_files=1029,source_manifest_sha256=c['source_manifest_sha256'],application=str(c['application']),application_sha256=c['application_sha256'],compiled_source_archive=c['compiled_source_archive'],entry_sha256=c['entry_sha256'],reference_jar_sha256=c['reference_jar_sha256'],profiles=profiles,recorded_owned_identities=64,recorded_owned_processes_all_terminal=True,observed_release_compiler_identities=len(c['observed_release_compilers']),observed_release_compiler_identities_terminal=True,actual_formal_sessions=0,GUI_verified=False,physical_audio_verified=False,full_current_regression_verified=False,whole_G06_accepted=False,whole_goal_complete=False,raw_archive_sha256=sha(raw),raw_files=hashes,observed_utc=datetime.now(timezone.utc).isoformat())
(run/'completion-observation.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({k:q[k] for k in ['status','original_session_id','profiles','source_manifest_sha256']},ensure_ascii=False))
