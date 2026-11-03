"""Read one completed profile while the other original profile may remain live."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,tomllib,zipfile

ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def archive(path,digest,files):
    assert sha(path)==digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(files) and set(z.namelist())==set(files)
        for name,h in files.items():
            with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h
def amd64(path):
    with Path(path).open('rb') as f:
        h=f.read(64);assert h[:2]==b'MZ';f.seek(int.from_bytes(h[60:64],'little'));h=f.read(26)
        assert h[:4]==b'PE\0\0' and h[4:6]==b'\x64\x86' and h[24:26]==b'\x0b\x02'
def function(text,name):
    start=text.index('fn '+name+'(');body=text.index('{',start);depth=0
    for i in range(body,len(text)):
        if text[i]=='{':depth+=1
        elif text[i]=='}':
            depth-=1
            if depth==0:return text[start:i+1]
    raise AssertionError('Missing function end')

ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',required=True,choices=['Debug','Release']);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-presentation-source-owner')
c=read(run/'cohort.json');d=run/a.profile;info=c['profiles'][a.profile]
assert info['compile_exit_code']==0 and info['version_exit_code']==0
assert set(info['products'])==set(info['tests'])=={'LiToModern','native_resource_loading','native_world_lighting_scene','native_render_environment','native_light_lifecycle','native_terrain_residence','native_painted_input','native_startup','native_login','native_world_entry'}
jobs=[p for p in c['process_history'] if Path(p['output']).is_relative_to(d)]
assert len(jobs)==13 and all(p['exit_code']==0 for p in jobs)
pins=read(run/'source-inputs.json');inputs=read(run/'compiled-inputs.json')
assert len(pins)==1024 and len(inputs)==1025 and all(inputs[n]==h for n,h in pins.items())
assert sha(run/'source-inputs.json')==c['source_manifest_sha256']
for name,h in inputs.items():assert sha(ROOT/name)==h,name
source=c['compiled_source_archive'];archive(source['path'],source['sha256'],inputs)
assert sha(ROOT/'啟動現代客戶端.ps1')==c['entry_sha256']
app=Path(c['application']);application=read(app/'application.json');before=read(app/'base-source-inputs.json')
assert [n for n in pins if pins[n]!=before[n]]==c['changed_sources'] and set(c['changed_sources'])==set(application['before'])
archive(app/'before.zip',application['before_archive_sha256'],application['before'])
context_v2=application.get('context_contract_version',1)>=2
if context_v2:
    red=read(app/'independent-regression-readback.json')
    assert red['original_session_id']==30212 and red['observed_terminal_exit_code']==1 and red['inner_test_exit_code']==101
    assert red['same_registered_assertion_preserved']
    with zipfile.ZipFile(red['raw_archive']) as z:
        assert 'old-root pixels survived the source-root change' in z.read('run/Release/test-native_world_lighting_scene/stdout.log').decode('utf-8-sig')
    with zipfile.ZipFile(app/'before.zip') as before_zip:
        before_source=before_zip.read('LiTo381-Modern/tests/native_world_lighting_scene.rs').decode('utf-8')
        assert function(before_source,'presentation_root_change_retires_old_model_and_source_leases')==function((PROJECT/'tests/native_world_lighting_scene.rs').read_text(encoding='utf-8'),'presentation_root_change_retires_old_model_and_source_leases')
else:
    red=read(app/'independent-red-readback.json')
    assert red['inner_test_exit_code']==101 and red['observed_terminal_exit_code']==0 and red['same_registered_assertion_preserved']
    with zipfile.ZipFile(red['raw_archive']) as z:
        assert 'Job reused an old-root Model' in z.read('run/stderr.log').decode('utf-8-sig')
assert red['source_manifest_sha256']==application['base_source_manifest_sha256'] and sha(red['raw_archive'])==red['raw_archive_sha256']
expected={'native_resource_loading':(12,0),'native_world_lighting_scene':(10,0),'native_render_environment':(4,0),'native_light_lifecycle':(1,0),'native_terrain_residence':(2,0),'native_painted_input':(2,0),'native_startup':(4,0),'native_login':(3,0),'native_world_entry':(2,2),'LiToModern':(17 if a.profile=='Debug' else 13,0)}
if context_v2:expected['native_world_lighting_scene']=(11,0)
messages=[json.loads(line) for line in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
artifacts={m['target']['name']:m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']}
assert set(artifacts)==set(expected)
names={};totals=dict(passed=0,failed=0,ignored=0)
for name,product in info['products'].items():
    exe=Path(product['path']).resolve(strict=True);assert exe.is_relative_to(PROJECT/'target/current')
    assert sha(exe)==product['sha256'] and Path(artifacts[name]['executable']).resolve()==exe
    assert artifacts[name]['features']==[] and artifacts[name]['profile']==product['compiler_profile'];amd64(exe)
    assert artifacts[name]['profile']['opt_level']==('z' if a.profile=='Release' else '0')
    text=(d/('test-'+name)/'stdout.log').read_text(encoding='utf-8-sig')
    rows=re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',text,re.M)
    assert rows and len(rows)==len({n for n,_ in rows})
    counts={k:sum(v==status for _,v in rows) for k,status in [('passed','ok'),('failed','FAILED'),('ignored','ignored')]}
    assert (counts['passed'],counts['ignored'])==expected[name] and counts['failed']==0
    terminal=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
    assert terminal==[('ok',str(counts['passed']),'0',str(counts['ignored']),'0','0')]
    assert all(info['tests'][name][key]==value for key,value in counts.items())
    for key,value in counts.items():totals[key]+=value
    names[name]=dict(rows)
assert names['native_resource_loading']['observed_archive_lifetime_denies_writes_to_all_four_sources']=='ok'
for test in ['shared_native_sources_release_only_after_the_last_owner_and_respect_limits','cancelled_shared_source_wait_does_not_retire_the_live_owner','concurrent_shared_native_archive_windows_match_independent_original_pixels','overlapping_world_models_reuse_source_leases_without_changing_pixels']:
    assert names['native_resource_loading'][test]=='ok'
assert names['native_resource_loading']['retained_archive_validation_and_failed_windows_keep_the_same_source_owner']=='ok'
assert names['native_resource_loading']['retained_native_geometry_keeps_sector_identity_locks_and_rejects_new_sources']=='ok'
assert names['native_terrain_residence']['original_siege563_residence_reloads_at_sector_boundary_and_retires_old_input']=='ok'
assert names['native_terrain_residence']['resident_native_city_pixels_match_complete_original_terrain']=='ok'
for test in ['presentation_sources_survive_scene_retirement_without_retaining_old_input','presentation_root_change_retires_old_model_and_source_leases','presentation_cancel_does_not_recapture_sources_from_a_pending_render','presentation_job_rejects_reused_model_from_other_resource_root']:
    assert names['native_world_lighting_scene'][test]=='ok'
if context_v2:assert names['native_world_lighting_scene']['presentation_resume_same_scene_rejects_retired_render_contexts']=='ok'
assert totals==info['counts']
tests=info['test_products_archive'];archive(tests['path'],tests['sha256'],{name+'.exe':v['sha256'] for name,v in info['products'].items()})
normal=info['normal'];assert normal['unique_entry_exit_code']==0
assert sha(normal['path'])==normal['sha256'];amd64(normal['path'])
archive(normal['archive']['path'],normal['archive']['sha256'],{'LiToModern.exe':normal['sha256']})
assert not (d/'version/stderr.log').read_bytes().strip() and b'LiToModern' in (d/'version/stdout.log').read_bytes()
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert (release['opt-level'],release['lto'],release['codegen-units'],release['strip'])==('z','thin',16,'symbols')
release_compiler_evidence_kind='not_applicable_Debug'
if a.profile=='Release':
    observed=read(run/'release-final-thinlto-observation.json')
    command=observed['CommandLine']
    assert all(flag in command for flag in ['-C opt-level=z','-C lto=thin','-C codegen-units=16','-C strip=symbols','--target x86_64-pc-windows-msvc'])
    if observed.get('retained_equivalent_products'):
        # Cargo may legitimately reuse every product after an unrelated test
        # list correction. Preserve the old compiler identity and prove exact
        # product equality instead of pretending that compiler ran again.
        proof=observed['retained_equivalent_products']
        assert application.get('test_only_correction') and application['runtime_source_changed_since_parent'] is False
        previous=Path(application['parent_qualified_related_run']).resolve(strict=True)
        assert previous.is_relative_to(ROOT/'驗證紀錄/current/native-presentation-source-owner')
        assert Path(proof['previous_run']).resolve()==previous
        pc=read(previous/'cohort.json');pq=read(previous/'Release/profile-qualified/qualified.json');pt=read(previous/'completion-observation.json')
        assert pc['source_manifest_sha256']==pq['source_manifest_sha256']==application['parent_qualified_source_manifest_sha256']
        assert pq['status']=='independently_qualified_presentation_source_owner_related_profile' and pq['counts']['failed']==0
        assert pt['observed_terminal_exit_code']==0 and pt['recorded_owned_processes_all_terminal']
        assert sha(previous/'Release/profile-qualified/qualified.json')==proof['previous_release_qualification_sha256']==pt['profiles']['Release']['qualification_sha256']
        assert sha(previous/'completion-observation.json')==proof['previous_completion_sha256']
        assert sha(previous/'release-final-thinlto-observation.json')==proof['previous_observation_sha256']==pq['raw_files']['Release-final-compiler-observation.json']
        original=read(previous/'release-final-thinlto-observation.json')
        assert {k:observed[k] for k in original}==original
        prior_inputs=read(previous/'compiled-inputs.json')
        changed=[n for n in inputs if inputs[n]!=prior_inputs[n]]
        assert set(prior_inputs)==set(inputs) and changed==application['canonical_change_since_parent']==[application['test_only_correction']['path']]
        assert changed==proof['changed_global_source_paths'] and changed==['LiTo381-Modern/tests/native_holy_weapon.rs']
        assert all(m['fresh'] for m in artifacts.values())
        assert {n:v['sha256'] for n,v in info['products'].items()}=={n:v['sha256'] for n,v in pc['profiles']['Release']['products'].items()}
        assert normal['sha256']==pq['normal_sha256']==pc['profiles']['Release']['normal']['sha256']
        archive(previous/'Release/profile-qualified/raw.zip',pq['raw_archive_sha256'],pq['raw_files'])
        release_compiler_evidence_kind='retained_prior_actual_compile_for_identical_executed_products'
    else:
        build_job=next(p for p in jobs if Path(p['output'])==d/'build')
        assert observed['ParentProcessId']==build_job['pid']
        assert datetime.fromisoformat(observed['CreationUtc'])>=datetime.fromisoformat(build_job['creation_utc'])
        release_compiler_evidence_kind='current_owned_compiler_observation'
terrain=read(d/'terrain-output/563-static.json');assert terrain['synthetic_authority_fixture']
assert terrain['first']['load_profile_us']['archive_sources_reused'] is False
assert terrain['before_boundary']['load_profile_us']['archive_sources_reused'] is True
assert terrain['after_boundary']['load_profile_us']['archive_sources_reused'] is True
assert terrain['first']['load_profile_us']['geometry_sources_reused'] is False
assert terrain['before_boundary']['load_profile_us']['geometry_sources_reused'] is True
assert terrain['after_boundary']['load_profile_us']['geometry_sources_reused'] is True
assert terrain['before_boundary']['geometry_sha256']==terrain['after_boundary']['geometry_sha256']
assert terrain['before_boundary']['native_terrain_center']!=terrain['after_boundary']['native_terrain_center']
for key in ['first','before_boundary','after_boundary']:
    assert terrain[key]['map_sectors']==19 and terrain[key]['static_resource_budget']['decoded_limit_bytes']==64<<20
    assert terrain[key]['static_resource_budget']['decoded_bytes']<=64<<20
ids=','.join(str(p['pid']) for p in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds())>0.001
out=d/'profile-qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'cohort-observation.json').write_bytes((run/'cohort.json').read_bytes());(out/'owned-CIM.json').write_bytes(p.stdout)
files={'qualifier.py':out/'qualifier.py','cohort-observation.json':out/'cohort-observation.json','owned-CIM.json':out/'owned-CIM.json'}
if a.profile=='Release':
    files['Release-final-compiler-observation.json']=run/'release-final-thinlto-observation.json'
for directory,prefix in [(d,a.profile+'/'),(app,'application/')]:
    for file in directory.rglob('*'):
        if file.is_file() and out not in file.parents and file.suffix.lower() not in ['.exe','.zip']:
            assert file.resolve().is_relative_to(directory) and not file.is_symlink()
            files[prefix+file.relative_to(directory).as_posix()]=file
raw=out/'raw.zip';hashes={name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(raw,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,p in files.items():z.write(p,name)
archive(raw,sha(raw),hashes)
result=dict(status='independently_qualified_presentation_source_owner_related_profile',profile=a.profile,
    release_compiler_evidence_kind=release_compiler_evidence_kind,
    source_publication_context_tests_passed=context_v2,
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=c['source_manifest_sha256'],
    compiled_source_archive_sha256=source['sha256'],changed_sources=c['changed_sources'],
    actual_test_executables=10,counts=totals,all_test_results=names,test_products_archive_sha256=tests['sha256'],
    normal_sha256=normal['sha256'],normal_product_archive_sha256=normal['archive']['sha256'],
    previous_source_failures_not_relabelled=True,shared_source_owner_lease_tests_passed=True,normal_presentation_source_lifetime_tests_passed=True,old_root_red_assertion_now_passed=True,
    recorded_profile_owned_identities=len(jobs),recorded_profile_owned_processes_terminal=True,
    initial_archive_pins_us=terrain['first']['load_profile_us']['archive_pins_us'],
    reload_archive_pins_us={k:terrain[k]['load_profile_us']['archive_pins_us'] for k in ['before_boundary','after_boundary']},
    raw_archive_sha256=sha(raw),raw_files=hashes,actual_formal_sessions=0,new_siege_case_verified=False,
    GUI_verified=False,full_current_regression_verified=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({k:result[k] for k in ['status','profile','counts','normal_sha256','initial_archive_pins_us','reload_archive_pins_us']},ensure_ascii=False))
