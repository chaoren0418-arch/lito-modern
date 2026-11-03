"""Independently qualify one current normal image and the directly affected tests."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,struct,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):
    for attempt in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def image(v):
    assert sha(v['path'])==v['sha256']
    with Path(v['path']).open('rb') as f:
        assert f.read(2)==b'MZ';f.seek(0x3c);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
    archive=v['archive'];assert sha(archive['path'])==archive['sha256']
    with zipfile.ZipFile(archive['path']) as z:
        for n,h in archive['files'].items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h==v['sha256']
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],required=True);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-lawful');c=read(run/'cohort.json');p=a.profile;info=c['profiles'][p]
assert set(info['targets'])=={'native_skills','native_skill_selection','native_attack_skills'} and all('counts' in v for v in info['targets'].values())
assert c['source_files']==1030 and c['compiled_input_files']==1031 and tuple(c['release_configuration'][k] for k in ['opt-level','lto','strip','codegen-units'])==('z','thin','symbols',16)
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1031
identity_manifest=Path(c['application'])/'applied-source-inputs.json';assert sha(identity_manifest)==c['source_manifest_sha256'] and read(run/'source-inputs.json')==read(identity_manifest)
for n,h in inputs.items():assert sha(ROOT/n)==h,n
app=Path(c['application']);assert sha(app/'application.json')==c['application_sha256'];application=read(app/'application.json');assert application['source_manifest_sha256']==c['source_manifest_sha256']
assert len(application['after'])==2
for n,h in application['after'].items():assert sha(ROOT/n)==h
assert sha(c['normal_entry'])==c['entry_sha256'];image(info['normal']);assert info['normal']['unique_entry_exit_code']==info['normal']['version_exit_code']==0
archive=c['compiled_source_archive'];assert archive['files']==inputs and sha(archive['path'])==archive['sha256']
with zipfile.ZipFile(archive['path']) as z:
    assert set(z.namelist())==set(inputs)
    for n,h in inputs.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
old=read(ROOT/'驗證紀錄/current/native-default111/20261017-135059-659904/cohort.json')['profiles'][p];assert sha(old['qualification'])==old['qualification_sha256']
new={'disintegrate_request_contract::'+n for n in ['current_c02_exact_object_coordinates_and_readonly_authority','range_ten_boundary_and_target_changes_use_current_received_coordinates','invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests','mp_and_lawful_are_server_permission_inputs_and_never_locally_spent','selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection']}
new.add('captured_c02_disintegrate_lawful352_is_a_rejection_without_authority_or_cast')

total=0
for target,v in info['targets'].items():
    image(v);folder=run/p/target;inventory=set(re.findall(r'^(.+): test$',(folder/'inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M));ignored=set(re.findall(r'^(.+): test$',(folder/'ignored-inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M))
    previous=set(re.findall(r'^(.+): test$',(Path(old['run'])/'cases'/target/'inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M));expected=previous|(new if target=='native_attack_skills' else set())
    assert inventory==expected==set(v['test_names']) and not ignored
    text=(folder/'tests/stdout.log').read_text(encoding='utf-8-sig');assert set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))==expected
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('ok',str(len(expected)),'0','0','0','0')]
    assert v['counts']==dict(passed=len(expected),failed=0,ignored=0,measured=0,filtered_out=0);total+=len(expected)
assert total==31
jobs=[j for j in c['process_history'] if j['label'].startswith(p+'/')];assert len(jobs)==12 and all(j['exit_code']==0 for j in jobs)
ids=','.join(str(j['pid']) for j in jobs);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/(p+'-qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());save(out/'cohort-at-qualification.json',c)
files=[(out/'cohort-at-qualification.json','cohort.json')]
for job in jobs:files.extend((f,(Path(p)/Path(job['label']).relative_to(p)/f.name).as_posix()) for f in (run/job['label']).iterdir() if f.is_file())
pins={n:sha(f) for f,n in files}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for f,n in files:z.write(f,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(identity_manifest=str(identity_manifest),identity_manifest_sha256=sha(identity_manifest),runner_source_copy_sha256=sha(run/'source-inputs.json'),source_copy_serialization_differs=sha(run/'source-inputs.json')!=sha(identity_manifest),status='independently_qualified_native77_request_normal_related_profile',profile=p,run=str(run),source_manifest_sha256=c['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,normal=info['normal'],targets=info['targets'],
    passed=31,failed=0,ignored=0,added_contract_tests=1,original_registered_tests_preserved=True,recorded_owned_processes=jobs,recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),
    qualification_scope='current normal build/version0 and three directly affected request/selection/attack targets only',full_library_verified=False,full_G06_verified=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),sha256=sha(out/'qualified.json'),profile=p,normal_sha256=info['normal']['sha256'],passed=31,whole_goal_complete=False),ensure_ascii=False))
