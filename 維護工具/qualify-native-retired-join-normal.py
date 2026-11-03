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
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-retired-join-normal')
c=read(run/'cohort.json');info=c['profiles'][p];app=Path(c['application']);application=read(app/'application.json')
assert sha(app/'application.json')==c['application_sha256'] and application['source_manifest_sha256']==c['source_manifest_sha256']
assert application['applied'] and len(application['after_files'])==3
for name,digest in application['after_files'].items():assert sha(ROOT/name)==digest
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1041 and len(read(run/'source-inputs.json'))==1040
assert sha(run/'source-inputs.json')==c['source_manifest_sha256']
for name,digest in inputs.items():assert sha(ROOT/name)==digest
archive=c['compiled_source_archive'];assert archive['files']==inputs and sha(archive['path'])==archive['sha256']
with zipfile.ZipFile(archive['path']) as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in inputs.items())
assert sha(c['normal_entry'])==c['entry_sha256'] and info['normal']['unique_entry_exit_code']==info['normal']['version_exit_code']==0
image(info['normal']);assert tuple(c['release_configuration'][k] for k in ['opt-level','lto','strip','codegen-units'])==('z','thin','symbols',16)
targets={'native_skills','native_skill_selection','native_attack_skills','native_animation','native_motion','native_water_clock','native_render_environment','native_cone_cold','native_room','native_room_departures','native_room_force'}
assert set(info['targets'])==targets
prior=read(ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950/cohort.json')['profiles'][p]
room_prior={'Debug':ROOT/'驗證紀錄/current/default-native-regression/20261018-032357-157-Debug','Release':ROOT/'驗證紀錄/current/default-native-regression/20261018-033903-037-Release'}
cold_names={'cold_request_uses_observed_C14_and_six_cell_range_without_spending_authority','cold_raw_S23_is_atomic_and_defers_hurt_until_effect_arrival','cold_selector_rechecks_target_life_generation_owner_and_book'}
total=0;old_related=0
for target,v in info['targets'].items():
    image(v);folder=run/p/target
    names=set(re.findall(r'^(.+): test$',(folder/'inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M))
    ignored=set(re.findall(r'^(.+): test$',(folder/'ignored-inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M));assert names==set(v['test_names']) and not ignored
    if target in prior['targets']:
        assert names==set(prior['targets'][target]['test_names']);old_related+=len(names)
    if target in ['native_room','native_room_departures','native_room_force']:
        expected=set(re.findall(r'^(.+): test$',(room_prior[p]/'cases'/target/'inventory/stdout.log').read_text('utf-8-sig'),re.M));assert names==expected
    if target=='native_cone_cold':assert names==cold_names
    if target=='native_animation':assert 'hurt_queued_after_motion::eight_directions_hurt_waits_for_walk_without_camera_rewind_and_old_life_replay' in names
    text=(folder/'tests/stdout.log').read_text(encoding='utf-8-sig')
    assert set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))==names
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('ok',str(len(names)),'0','0','0','0')]
    assert v['counts']==dict(passed=len(names),failed=0,ignored=0,measured=0,filtered_out=0);total+=len(names)
assert old_related==57
selected=info['library_selected'];image(selected);assert set(selected['groups'])=={'pending_join','wire_spatial'}
unit_total=0
for group,v in selected['groups'].items():
    folder=run/p/('library-'+group)
    names=set(re.findall(r'^(.+): test$',(folder/'inventory/stdout.log').read_text('utf-8-sig'),re.M))
    ignored=set(re.findall(r'^(.+): test$',(folder/'ignored-inventory/stdout.log').read_text('utf-8-sig'),re.M))
    assert names==set(v['test_names'])and not ignored and len(names)==(2 if group=='pending_join'else 18)
    if group=='pending_join':assert names=={'modern_lito::native_login::pending_join_departure_tests::observed_early_departure_and_fourteen_adversarial_join_contracts','modern_lito::native_login::pending_join_departure_tests::retired_peer_notifications_cannot_preempt_pending_join_or_erase_new_roster'}
    assert all(name.startswith(v['prefix'])for name in names)
    text=(folder/'tests/stdout.log').read_text('utf-8-sig');assert set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))==names
    rows=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
    assert len(rows)==1 and rows[0][:5]==('ok',str(len(names)),'0','0','0')
    assert v['counts']==dict(zip(['passed','failed','ignored','measured','filtered_out'],map(int,rows[0][1:])))
    unit_total+=len(names)
assert unit_total==20
restoration=read(run/p/'protected-outputs/restoration.json');assert restoration['existing_files_restored']
for n,h in restoration['before_files'].items():assert sha(ROOT/'LiTo381-Modern/target/current/verification'/n)==h
archive_path=run/p/'protected-outputs/generated.zip';assert sha(archive_path)==restoration['generated_archive_sha256']
with zipfile.ZipFile(archive_path)as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in restoration['generated_files'].items())
jobs=[j for j in c['process_history'] if j['label'].startswith(p+'/')];assert len(jobs)==43 and all(j['exit_code']==0 for j in jobs)
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
q=dict(status='independently_qualified_retired_join_normal_eleven_targets_and_selected_library',profile=p,run=str(run),source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,
    normal=info['normal'],targets=info['targets'],passed=total+unit_total,integration_passed=total,selected_library_passed=unit_total,selected_library=selected,failed=0,ignored=0,original_registered_tests_preserved=True,preserved_cold_contract_tests=3,preserved_pending_join_contract_test=1,pending_join_cases_in_test=14,preserved_wire_spatial_tests=18,added_retired_peer_contract_test=1,retired_peer_cases_in_test=4,
    recorded_owned_processes=jobs,recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),full_library_verified=False,full_G06_verified=False,actual_new_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=p,passed=total+unit_total,normal_sha256=info['normal']['sha256'],whole_goal_complete=False),ensure_ascii=False))
