"""Independently bind affected tests and both normal EXEs; no TCP/GUI promotion."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def verify_archive(path,hashes):
    with zipfile.ZipFile(path)as z:
        assert len(z.namelist())==len(set(z.namelist()))and set(z.namelist())==set(hashes)
        for name,digest in hashes.items():
            with z.open(name)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest,name
def pe64(path):
    with Path(path).open('rb')as f:
        assert f.read(2)==b'MZ';f.seek(60);offset=int.from_bytes(f.read(4),'little');f.seek(offset);h=f.read(26)
        assert h[:4]==b'PE\0\0'and h[4:6]==b'\x64\x86'and h[24:26]==b'\x0b\x02'
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-installed-resource-profile-normal')and a.session>0 and a.terminal_exit_code==0 and a.terminal_chunk not in ['','pending','unknown']
c=read(run/'cohort.json');assert c['status']=='executed_installed_resource_profile_normal_related_both_profiles'and c['owned_process']is None
assert len(c['process_history'])==10 and all(row['exit_code']==0 and not row.get('cleanup_stopped_original_owned_handle')for row in c['process_history'])
assert not c['source_overlays_used']and not c['diagnostic_API_extension']and c['normal_product_build']and c['actual_sessions']==0
app=Path(c['application']);application=read(app/'application.json');inputs=read(run/'compiled-inputs.json');source=read(run/'source-inputs.json');manifest=sha(run/'source-inputs.json')
assert len(source)==1048 and len(inputs)==1049 and manifest==c['source_manifest_sha256']==application['source_manifest_sha256']==sha(app/'applied-source-inputs.json')
assert source==read(app/'applied-source-inputs.json')and inputs==read(app/'applied-compiled-inputs.json')
for name,digest in inputs.items():assert sha(ROOT/name)==digest,name
expected={n for n in source if n.startswith(('LiTo381-Modern/src/','LiTo381-Modern/tests/'))};actual={p.relative_to(ROOT).as_posix()for folder in ['src','tests']for p in(P/folder).rglob('*')if p.is_file()};assert actual==expected
candidate=read(application['candidate']);assert sha(application['candidate'])==application['candidate_sha256']
assert manifest=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
assert candidate['before_source_manifest_sha256']==application['before_source_manifest_sha256']=='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
assert candidate['candidate_source_manifest_sha256']==manifest and candidate['only_cfg_test_reference_changed'] and not candidate['Runtime_changed']
review=read(application['candidate_review']);assert sha(application['candidate_review'])==application['candidate_review_sha256'] and review['candidate_sha256']==application['candidate_sha256'] and review['exact_one_cfg_test_file'] and review['Runtime_unchanged'] and review['original_assertions_preserved']
old=read(app/'before-compiled-inputs.json');assert set(old)==set(inputs) and len(old)==1049
expected_changes={'LiTo381-Modern/src/modern_lito/resources.rs'};changed={n for n,h in inputs.items() if old[n]!=h};assert changed==set(application['changed_files'])==set(candidate['after_files'])==expected_changes
assert application['created_files']==[] and application['after_files']==candidate['after_files'] and not application['normal_product_source_modified'] and application['test_source_modified'] and not application['clocks_changed']
verify_archive(app/'before.zip',application['before_files']);assert sha(app/'before.zip')==application['before_archive_sha256'];verify_archive(app/'applied-payload.zip',application['after_files']);assert sha(app/'applied-payload.zip')==application['payload_archive_sha256']
with zipfile.ZipFile(app/'before.zip') as z:baseline=z.read(next(iter(expected_changes))).decode('utf-8').replace('\r\n','\n')
current=(ROOT/next(iter(expected_changes))).read_text('utf-8-sig');boundary=baseline.index('#[cfg(test)]');assert baseline[:boundary]==current[:boundary] and all(line in current for line in baseline.splitlines() if 'assert' in line)
proof=read(review['component_qualification']);assert sha(review['component_qualification'])==review['component_qualification_sha256'] and proof['qualified'] and proof['candidate_sha256']==application['candidate_sha256'] and proof['recorded_original_owned_processes_terminal'] and proof['original_assertions_preserved'] and not proof['Runtime_changed']
comparison=read(candidate['comparison']);assert sha(candidate['comparison'])==candidate['comparison_sha256'] and comparison['complete_record_multiset_equal'] and comparison['records']==37497 and len(comparison['changed_ordered_rows'])==192 and comparison['pak_unchanged']
prior=read(application['before_normal_qualification']);assert sha(application['before_normal_qualification'])==application['before_normal_qualification_sha256'] and prior['source_manifest_sha256']==application['before_source_manifest_sha256'] and prior['passed_per_profile']==19 and prior['recorded_owned_processes_all_terminal']
history=application['qualification_before_repair'];assert sha(history['state'])==history['state_sha256'];negative={'qualification':review['component_qualification']}
release=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['profile']['release'];assert tuple(release[k]for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
required={'same_endpoint_catalog_rebinds_only_matching_routes_once_after_fresh_game_character','cross_endpoint_or_port_change_never_carries_source_rental_routes','old_or_skipped_connection_and_wrong_character_consume_the_catalog','absent_directory_and_routes_for_another_peer_do_not_invent_rental_service','changing_the_pending_owner_or_destination_cannot_retarget_a_captured_catalog','remembered_directory_is_an_owned_snapshot_and_connection_overflow_cannot_wrap'}
profiles={};files={name:run/name for name in ['runner.py','cohort.json','source-inputs.json','compiled-inputs.json']};files['qualifier.py']=Path(__file__);files.update({'application.json':app/'application.json','candidate.json':Path(application['candidate']),'candidate-review.json':Path(application['candidate_review']),'pre-apply-text-reference-components.json':Path(negative['qualification'])})
for profile in ['Debug','Release']:
    d=run/profile;info=c['profiles'][profile];assert set(info['targets'])=={'native_teleport','native_services','native_login'}
    normal=Path(info['normal']['path']);assert sha(normal)==info['normal']['sha256'];pe64(normal)
    entry_jobs=[row for row in c['process_history']if row['output']==str(d/'normal-build')];assert len(entry_jobs)==1 and entry_jobs[0]['command']==['pwsh','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile]
    cargo_jobs=[row for row in c['process_history']if row['output']==str(d/'test-build')];assert len(cargo_jobs)==1
    expected_args=['test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--test','native_teleport','--test','native_services','--test','native_login']+(['--release']if profile=='Release'else[])
    assert cargo_jobs[0]['command'][1:]==expected_args
    artifacts=[]
    for line in(d/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
        if not line.startswith('{'):continue
        row=json.loads(line)
        if row.get('reason')=='compiler-artifact'and row.get('executable')and row['profile']['test']:artifacts.append(row)
    assert len(artifacts)==3 and {row['target']['name']for row in artifacts}==set(info['targets'])
    members={'LiToModern.exe':info['normal']['sha256']};counts={}
    for name,target in info['targets'].items():
        artifact=next(row for row in artifacts if row['target']['name']==name);exe=Path(target['path']);assert Path(artifact['executable']).resolve()==exe.resolve()and artifact['features']==[];pe64(exe);assert sha(exe)==target['sha256']
        rows=[row for row in c['process_history']if row['output']==str(d/name)];assert len(rows)==1 and rows[0]['command']==[str(exe),'--test-threads=1']
        log=(d/name/'stdout.log').read_text('utf-8-sig');names=re.findall(r'^test (.+) \.\.\. ok$',log,re.M);assert len(names)==len(set(names))and names==target['test_names']
        total={'native_teleport':14,'native_services':2,'native_login':3}[name];assert len(names)==total and target['counts']==dict(passed=total,failed=0,ignored=0)
        assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)==[(str(total),'0','0','0','0')]
        if name=='native_teleport':assert required<=set(names)
        members[name+'.exe']=target['sha256'];counts[name]=target['counts']
    assert sha(d/'products.zip')==info['product_archive_sha256'];verify_archive(d/'products.zip',members)
    profiles[profile]=dict(normal=info['normal'],targets=info['targets'],passed=19,failed=0,ignored=0,product_archive=str(d/'products.zip'),product_archive_sha256=sha(d/'products.zip'))
    for p in d.rglob('*'):
        if p.is_file()and p.suffix!='.zip':files[profile+'/'+p.relative_to(d).as_posix()]=p
identities=[c['runner'],*c['process_history']];filters=' OR '.join('ProcessId='+str(pid)for pid in sorted({row['pid']for row in identities}));cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
z=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert z.returncode==0 and not z.stderr;live=json.loads(z.stdout.decode('utf-8-sig'))if z.stdout.strip()else[];live=[live]if isinstance(live,dict)else live
for row in live:
    for identity in identities:
        if row['ProcessId']==identity['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(identity['creation_utc'])).total_seconds())>.001
out=run/'qualified';out.mkdir();(out/'owned-CIM.json').write_bytes(z.stdout);files['owned-CIM.json']=out/'owned-CIM.json';pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,p in files.items():z.write(p,n)
verify_archive(out/'raw.zip',pins)
with zipfile.ZipFile(out/'compiled-source.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n in inputs:z.write(ROOT/n,n)
verify_archive(out/'compiled-source.zip',inputs)
q=dict(status='independently_qualified_installed_resource_profile_normal_related_both_profiles',source_manifest_sha256=manifest,source_files=1048,compiled_input_files=1049,application=str(app),cohort=str(run),cohort_sha256=sha(run/'cohort.json'),profiles=profiles,normal_mains={p:x['normal']['sha256']for p,x in profiles.items()},passed_per_profile=19,new_behavior_tests=0,existing_protocol_smoke_tests_per_profile=19,isolated_owned_focus_API_conditions=0,rental_entry_guard_actual_wire_verified=False,hotel_return_GUI_verified=False,new_optin_test_compiled_by_this_scope=False,original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,recorded_owned_identities=len(identities),recorded_owned_processes_all_terminal=True,recorded_identities=identities,compiled_source_archive=str(out/'compiled-source.zip'),compiled_source_archive_sha256=sha(out/'compiled-source.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,source_overlays_used=False,diagnostic_API_extension=False,normal_product_build=True,actual_sessions=0,current75_same_endpoint_verified=False,full_broad_regression_verified=False,full_library_verified=False,GUI_verified=False,whole_G06_complete=False,whole_goal_complete=False,qualified_utc=datetime.now(timezone.utc).isoformat())
save(out/'qualified.json',q);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(qualification=str(out/'qualified.json'),profiles={p:dict(passed=19,failed=0)for p in profiles},terminal_identities=len(identities),actual_sessions=0),ensure_ascii=False))
