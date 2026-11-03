"""Observe actual pipeline exit and both independent related qualifications."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def archive(path,digest,files):
    assert sha(path)==digest
    with zipfile.ZipFile(path) as z:
        assert set(z.namelist())==set(files)
        for n,h in files.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native544-integration')
c=read(run/'cohort.json');assert c['status']=='executed_current_exact544_normals_and_sprite_scopes' and c['owned_process'] is None
assert len(c['process_history'])==14 and all(j['exit_code']==0 for j in c['process_history'])
terminal=read(run/'launch-exit.json');binding=read(run/'original-session-binding.json');own=read(run/'original-runner-process.json')
assert terminal['observed_terminal_exit_code']==0 and terminal['terminal_tool_chunk_id']
assert terminal['original_session_id']==binding['original_session_id'] and binding['runner_identity']==own
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1029
for n,h in inputs.items():assert sha(ROOT/n)==h
assert sha(run/'source-inputs.json')==c['source_manifest_sha256'] and len(read(run/'source-inputs.json'))==1028
source=c['compiled_source_archive'];assert source['files']==inputs;archive(source['path'],source['sha256'],inputs)
profiles={}
files={'cohort.json':run/'cohort.json','launch-exit.json':run/'launch-exit.json','original-runner-process.json':run/'original-runner-process.json','original-session-binding.json':run/'original-session-binding.json','observer.py':Path(__file__)}
for profile in ['Debug','Release']:
    qp=run/profile/'qualified/qualified.json';q=read(qp)
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['recorded_profile_jobs_all_terminal'] and q['unique_sprite_tests_executed']==19
    archive(qp.parent/'raw.zip',q['raw_archive_sha256'],q['raw_files'])
    for kind,member in [('normal','LiToModern.exe'),('test','lito_modern.exe')]:
        info=c['profiles'][profile][kind];assert sha(info['path'])==info['sha256']==q['normal_sha256' if kind=='normal' else 'test_sha256']
        retained=info['archive'];archive(retained['path'],retained['sha256'],{member:info['sha256']})
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),normal_sha256=q['normal_sha256'],test_sha256=q['test_sha256'],unique_sprite_tests_executed=19)
    files[profile+'/qualification.json']=qp
identities=[(own['pid'],own['creation_utc'])]+[(j['pid'],j['creation_utc']) for j in c['process_history']]+[(j['ProcessId'],j['created']) for j in c['observed_compilers']]
identities=sorted(set(identities));ids=','.join(str(p) for p,t in identities)
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for pid,created in identities:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())>.001
assert not (run/'completion-observation.json').exists()
(run/'completion-CIM.json').write_bytes(raw);files['completion-CIM.json']=run/'completion-CIM.json';hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(run/'completion-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
archive(run/'completion-raw.zip',sha(run/'completion-raw.zip'),hashes)
q=dict(status='independently_observed_current_exact544_normals_and_sprite_both_profiles',source_manifest_sha256=c['source_manifest_sha256'],source_files=1028,compiled_input_files=1029,
    original_session_id=binding['original_session_id'],observed_terminal_exit_code=0,profiles=profiles,recorded_owned_identities=len(identities),recorded_owned_all_terminal=True,
    unique_sprite_test_bodies_per_profile=19,actual_formal_sessions=0,new_casts=0,full_library_test_bodies_executed=False,full_G06_verified=False,GUI_verified=False,physical_audio_verified=False,
    whole_goal_complete=False,raw_files=hashes,raw_archive_sha256=sha(run/'completion-raw.zip'))
(run/'completion-observation.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(status=q['status'],normal_mains={p:v['normal_sha256'] for p,v in profiles.items()},recorded_owned_identities=len(identities)),ensure_ascii=False))
