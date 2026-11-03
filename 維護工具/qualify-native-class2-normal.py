"""Independently read one exact-source normal image and seven related test images."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def image(v):
    assert sha(v['path'])==v['sha256']
    with Path(v['path']).open('rb') as f:
        assert f.read(2)==b'MZ';f.seek(0x3c);at=struct.unpack('<I',f.read(4))[0];f.seek(at);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
    archive=v['archive'];assert sha(archive['path'])==archive['sha256']
    with zipfile.ZipFile(archive['path']) as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h==v['sha256'] for n,h in archive['files'].items())
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],required=True);a=ap.parse_args();run=a.run.resolve(strict=True);p=a.profile
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-class2-normal')
c=read(run/'cohort.json');info=c['profiles'][p];app=Path(c['application']);application=read(app/'application.json')
assert sha(app/'application.json')==c['application_sha256'] and application['source_manifest_sha256']==c['source_manifest_sha256']
assert application['applied'] and len(application['after_files'])==16
for name,digest in application['after_files'].items():assert sha(ROOT/name)==digest
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1035 and len(read(run/'source-inputs.json'))==1034
assert sha(run/'source-inputs.json')==c['source_manifest_sha256']
for name,digest in inputs.items():assert sha(ROOT/name)==digest
archive=c['compiled_source_archive'];assert archive['files']==inputs and sha(archive['path'])==archive['sha256']
with zipfile.ZipFile(archive['path']) as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in inputs.items())
assert sha(c['normal_entry'])==c['entry_sha256'] and info['normal']['unique_entry_exit_code']==info['normal']['version_exit_code']==0
image(info['normal']);assert tuple(c['release_configuration'][k] for k in ['opt-level','lto','strip','codegen-units'])==('z','thin','symbols',16)
targets={'native_skills','native_skill_selection','native_attack_skills','native_animation','native_motion','native_water_clock','native_render_environment'}
assert set(info['targets'])==targets
prior=read(ROOT/'驗證紀錄/current/native77-lawful/validation-20261017-195458-430865/cohort.json')['profiles'][p]
new={'call_lightning_request_contract::'+name for name in ['current_c02_exact_object_coordinates_and_readonly_authority','range_six_boundary_and_target_changes_use_current_received_coordinates','invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests','mp_and_lawful_are_server_permission_inputs_and_never_locally_spent','selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection']}
total=0;old_related=0
for target,v in info['targets'].items():
    image(v);folder=run/p/target
    names=set(re.findall(r'^(.+): test$',(folder/'inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M))
    ignored=set(re.findall(r'^(.+): test$',(folder/'ignored-inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M));assert names==set(v['test_names']) and not ignored
    if target in prior['targets']:
        assert names==set(prior['targets'][target]['test_names'])|(new if target=='native_attack_skills' else set());old_related+=len(names)
    if target=='native_animation':assert 'hurt_queued_after_motion::eight_directions_hurt_waits_for_walk_without_camera_rewind_and_old_life_replay' in names
    text=(folder/'tests/stdout.log').read_text(encoding='utf-8-sig')
    assert set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))==names
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('ok',str(len(names)),'0','0','0','0')]
    assert v['counts']==dict(passed=len(names),failed=0,ignored=0,measured=0,filtered_out=0);total+=len(names)
assert old_related==36
jobs=[j for j in c['process_history'] if j['label'].startswith(p+'/')];assert len(jobs)==24 and all(j['exit_code']==0 for j in jobs)
ids=','.join(str(j['pid']) for j in jobs);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/(p+'-qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();save(out/'cohort-at-qualification.json',c);(out/'reader.py').write_bytes(Path(__file__).read_bytes())
files=[(out/'cohort-at-qualification.json','cohort.json')]+[(f,f.relative_to(run).as_posix()) for job in jobs for f in (run/job['label']).iterdir() if f.is_file()]
pins={name:sha(path) for path,name in files}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for path,name in files:z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_class2_normal_seven_related_targets',profile=p,run=str(run),source_manifest_sha256=c['source_manifest_sha256'],source_files=1034,compiled_input_files=1035,
    normal=info['normal'],targets=info['targets'],passed=total,failed=0,ignored=0,original_registered_tests_preserved=True,added_contract_tests=5,added_eight_direction_animation_test=1,
    recorded_owned_processes=jobs,recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),full_library_verified=False,full_G06_verified=False,actual_new_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=p,passed=total,normal_sha256=info['normal']['sha256'],whole_goal_complete=False),ensure_ascii=False))
