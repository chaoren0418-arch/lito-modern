"""Independent current-source native77 normal/profile qualification, no GUI claims."""
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
def pe(p):
    with Path(p).open('rb') as f:
        assert f.read(2)==b'MZ';f.seek(0x3c);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0';assert struct.unpack('<H',f.read(2))[0]==0x8664
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',required=True,choices=['Debug','Release']);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-scene-integration')
c=read(run/'cohort.json');p=a.profile;info=c['profiles'][p]
assert info['actual_asset_lifecycle']==dict(passed=10,failed=0,ignored=0,measured=0,filtered_out=2416)
assert len(info['selections'])==5 and info['registered_total']==2426 and info['unchanged_registered_ignored']==483
assert len(info['absent_library_test_modules'])==5
assert len(info['integration_targets'])==12
assert c['source_files']==1030 and c['compiled_input_files']==1031
assert tuple(c['release_configuration'][k] for k in ['opt-level','lto','strip','codegen-units'])==('z','thin','symbols',16)
app=Path(c['application']);app_state=read(app/'application.json');assert sha(app/'application.json')==c['application_sha256']
assert app_state['applied'] and app_state['source_manifest_sha256']==c['source_manifest_sha256']==sha(run/'source-inputs.json')
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1031
for n,h in inputs.items():assert sha(ROOT/n)==h,n
assert sha(c['normal_entry'])==c['entry_sha256']
for file in app_state['after']:assert sha(ROOT/'LiTo381-Modern/src/modern_lito'/file)==app_state['after'][file]
for name in ['normal','test']:
    v=info[name];assert sha(v['path'])==v['sha256'];pe(v['path']);archive=v['archive'];assert sha(archive['path'])==archive['sha256']
    with zipfile.ZipFile(archive['path']) as z:
        for member,h in archive['files'].items():
            with z.open(member) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h==v['sha256']
source=c['compiled_source_archive'];assert sha(source['path'])==source['sha256'] and source['files']==inputs
with zipfile.ZipFile(source['path']) as z:
    assert set(z.namelist())==set(inputs)
    for n,h in inputs.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
def log(label):return (run/p/label/'stdout.log').read_text(encoding='utf-8-sig')
inventory=set(re.findall(r'^(.+): test$',log('inventory'),re.M));ignored=set(re.findall(r'^(.+): test$',log('ignored-inventory'),re.M))
original=read(Path(c['original_inventory_cohort'])/p/'inventory-audit/inventory.json');assert original['total_tests']==2410 and original['ignored_tests']==473
probe=['actual_budget_and_late_timer_do_not_catch_up','actual_cancel_and_failed_source_old_publication_cannot_relaunch','actual_canvas_screen_then_multiply_and_projection_preflight',
 'actual_fragment_submission_uses_screen_multiply_and_remains_click_through','actual_frozen_position_target_life_and_caster_retirement','actual_marker_timing_once_normal_and_critical',
 'actual_model_rebuild_restores_only_admission_not_pixels_or_callbacks','actual_prepare_generation_and_failed_source_retire_before_draw','actual_prepared_payload_publication_and_history_once',
 'actual_stale_generation_owner_and_life_stamps_never_rebind','cancellation_before_io_retires_and_graphics_are_explicit','canonical_hurt_waits_for_effect_while_legacy_fallback_is_preserved',
 'capture_rejects_noncast_nonhit_zero_world_dead_and_old_life','capture_selects_current_life_even_when_stale_tuple_precedes_it']
new={'modern_lito::native_disintegrate_tests::native77_probe::'+name for name in probe}|{'modern_lito::disintegrate_assets::tests::current_tw_namespace_requires_exact_profile_and_cancels_before_io'}
assert inventory==set(original['all_test_names'])|new|{'modern_lito::sprite::tests::reviewed544_marker_preserves_slots_and_rejects_corruption'}
assert ignored==set(original['ignored_test_names'])|{n for n in new if '::actual_' in n}
assert len(inventory)==2426 and len(ignored)==483
passed=set();skipped=set();selected_logs=[]
for prefix,v in info['selections'].items():
    label=prefix.replace('modern_lito::','').replace('::','-').strip('-');selected_logs.append(label)
    expected={n for n in inventory if prefix in n};assert sorted(expected)==v['test_names']
    rows=dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log(label),re.M));assert set(rows)==expected
    assert {n for n,status in rows.items() if status=='ok'}==expected-ignored
    assert {n for n,status in rows.items() if status=='ignored'}==expected&ignored
    counts=v['counts'];assert counts==dict(passed=len(expected-ignored),failed=0,ignored=len(expected&ignored),measured=0,filtered_out=len(inventory)-len(expected))
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log(label))==[('ok',*[str(counts[k]) for k in ['passed','failed','ignored','measured','filtered_out']])]
    assert not passed&expected;passed|=expected-ignored;skipped|=expected&ignored
actual={n for n in new if '::actual_' in n};rows=dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log('native77-actual-assets'),re.M));assert set(rows)==actual and set(rows.values())=={'ok'}
assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log('native77-actual-assets'))==[('ok','10','0','0','0','2416')]
resource=read(run/'resource-inputs-before-tests.json');assert resource['before_actual_asset_jobs'] and not any('actual-assets' in j['label'] for j in resource['pipeline_history_at_pin'])
resource_job=next(j for j in c['process_history'] if j['label']==p+'/native77-actual-assets');assert datetime.fromisoformat(resource['observed_utc'])<datetime.fromisoformat(resource_job['creation_utc'])
for n,h in resource['original_input_pins'].items():assert sha(n)==h and Path(n).stat().st_mtime_ns==resource['observed_input_mtimes'][n]
for n,m in resource['original_PAK_metadata'].items():assert (Path(n).stat().st_size,Path(n).stat().st_mtime_ns)==(m['bytes'],m['mtime_ns'])
jar=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';assert sha(jar)==c['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
integration_passed=0;integration_ignored=0;integration_details={}
old_broad=read(ROOT/'驗證紀錄/current/native-default111/20261017-135059-659904/cohort.json')['profiles'][p]
assert sha(old_broad['qualification'])==old_broad['qualification_sha256']
assert set(info['integration_targets'])=={'native_world','native_combat','native_attack_skills','native_animation','native_effect_lifecycle','native_world_effects','native_world_layers','native_lifecycle','native_skill_selection','native_chill_touch','native_selector_only','native_world_lighting_scene'}
for target,item in info['integration_targets'].items():
    assert sha(item['path'])==item['sha256'];pe(item['path']);assert sha(item['archive']['path'])==item['archive']['sha256']
    with zipfile.ZipFile(item['archive']['path']) as z:
        with z.open(target+'.exe') as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==item['sha256']
    current_names=set(re.findall(r'^(.+): test$',log(target+'/inventory'),re.M))
    current_ignored=set(re.findall(r'^(.+): test$',log(target+'/ignored-inventory'),re.M))
    assert sorted(current_names)==item['registered_tests'] and sorted(current_ignored)==item['registered_ignored']
    old_run=Path(old_broad['run'])/'cases'/target
    old_names=set(re.findall(r'^(.+): test$',(old_run/'inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M))
    old_ignored=set(re.findall(r'^test (.+?) \.\.\. ignored(?:,.*)?$',(old_run/'runtime/stdout.log').read_text(encoding='utf-8-sig'),re.M))
    assert current_names==old_names and current_ignored==old_ignored
    rows=dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log(target+'/tests'),re.M))
    assert set(rows)==current_names and {n for n,v in rows.items() if v=='ok'}==current_names-current_ignored and {n for n,v in rows.items() if v=='ignored'}==current_ignored
    counts=item['counts'];assert counts==dict(passed=len(current_names-current_ignored),failed=0,ignored=len(current_ignored),measured=0,filtered_out=0)
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log(target+'/tests'))==[('ok',str(counts['passed']),'0',str(counts['ignored']),'0','0')]
    integration_passed+=counts['passed'];integration_ignored+=counts['ignored'];integration_details[target]=item
jobs=[j for j in c['process_history'] if j['label'].startswith(p+'/')];assert len(jobs)==48 and all(j['exit_code']==0 for j in jobs)
ids=','.join(str(j['pid']) for j in jobs)
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for old in jobs:
        if row['ProcessId']==old['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(old['creation_utc'])).total_seconds())>.001
out=run/(p+'-qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());save(out/'cohort-at-qualification.json',c)
files=[(run/'resource-inputs-before-tests.json','resource-inputs-before-tests.json'),(out/'cohort-at-qualification.json','cohort-at-qualification.json')]
for label in ['normal-build','version','lib-build','inventory','ignored-inventory',*selected_logs,'native77-actual-assets']:
    files.extend((f,(Path(p)/label/f.name).as_posix()) for f in (run/p/label).iterdir() if f.is_file())
for target in info['integration_targets']:
    for label in ['inventory','ignored-inventory','tests']:
        files.extend((f,(Path(p)/target/label/f.name).as_posix()) for f in (run/p/target/label).iterdir() if f.is_file())
files.extend((f,(Path(p)/'integration-build'/f.name).as_posix()) for f in (run/p/'integration-build').iterdir() if f.is_file())
raw_pins={name:sha(file) for file,name in files}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for file,name in files:z.write(file,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    for n,h in raw_pins.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
q=dict(status='independently_qualified_current_native77_normal_related_profile',profile=p,run=str(run),source_manifest_sha256=c['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,
    normal=info['normal'],test=info['test'],registered_total=2426,registered_ignored=483,original_ignored_preserved=473,added_explicit_resource_optins=10,
    passed_related_bodies=len(passed),ignored_related_bodies=len(skipped),explicit_actual_asset_passed=10,total_distinct_executed=len(passed|actual),passed_related_test_names=sorted(passed),explicit_actual_asset_test_names=sorted(actual),
    selections=info['selections'],integration_targets=integration_details,external_integration_passed=integration_passed,external_integration_ignored=integration_ignored,
    recorded_jobs=jobs,recorded_owned_all_terminal=True,raw_files=raw_pins,raw_archive_sha256=sha(out/'raw.zip'),
    resource_input_pins=resource['original_input_pins'],qualification_scope='current_normal_build_and_native77_scene_related_library_bodies_only',
    real_audio_device_verified=False,new_casts=0,new_sessions=0,GUI_verified=False,full_G06_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q)
print(json.dumps(dict(qualification=str(out/'qualified.json'),sha256=sha(out/'qualified.json'),profile=p,passed_related_bodies=len(passed),explicit_actual_asset_passed=10,external_integration_passed=integration_passed,external_integration_ignored=integration_ignored,whole_goal_complete=False),ensure_ascii=False))
