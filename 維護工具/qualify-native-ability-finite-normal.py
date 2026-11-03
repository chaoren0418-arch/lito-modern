"""Qualify normal images, scene23, ability7 and original finite resources11; keep broad/library/live scopes separate."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,struct,subprocess,time,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def image(v):
    assert sha(v['path'])==v['sha256']
    with Path(v['path']).open('rb')as f:
        assert f.read(2)==b'MZ';f.seek(0x3c);at=struct.unpack('<I',f.read(4))[0];f.seek(at);assert f.read(4)==b'PE\0\0'and struct.unpack('<H',f.read(2))[0]==0x8664
    ar=v['archive'];assert sha(ar['path'])==ar['sha256']
    with zipfile.ZipFile(ar['path'])as z:
        for n,h in ar['files'].items():
            with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h==v['sha256']
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-ability-finite-normal');c=read(run/'cohort.json');profile=a.profile;info=c['profiles'][profile];app=Path(c['application']);application=read(app/'application.json')
assert application['applied']and len(application['after_files'])==13 and sha(app/'application.json')==c['application_sha256']
assert c['source_manifest_sha256']==application['source_manifest_sha256']==sha(run/'source-inputs.json')
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1048 and len(read(run/'source-inputs.json'))==1047
for n,h in inputs.items():assert sha(ROOT/n)==h,n
ar=c['compiled_source_archive'];assert ar['files']==inputs and sha(ar['path'])==ar['sha256']
with zipfile.ZipFile(ar['path'])as z:
    assert set(z.namelist())==set(inputs)
    for n,h in inputs.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
assert sha(c['normal_entry'])==c['entry_sha256']and info['normal']['unique_entry_exit_code']==info['normal']['version_exit_code']==0
assert tuple(c['release_configuration'][k]for k in['opt-level','lto','strip','codegen-units'])==('z','thin','symbols',16);image(info['normal'])
expected=dict(native_world_lighting_scene=(23,0),native_ability_buffs=(5,0),native_ability_world=(2,0),native_additional_finite_resources=(1,1),native_ability_live=(1,1))
assert set(info['targets'])==set(expected)
for name,(total,ignored_count)in expected.items():
    item=info['targets'][name];image(item);folder=run/profile/name
    names=set(re.findall(r'^(.+): test$',(folder/'inventory/stdout.log').read_text('utf-8-sig'),re.M));ignored=set(re.findall(r'^(.+): test$',(folder/'ignored-inventory/stdout.log').read_text('utf-8-sig'),re.M))
    assert names==set(item['test_names'])and len(names)==total and len(ignored)==ignored_count and ignored<=names
    text=(folder/'tests/stdout.log').read_text('utf-8-sig');assert set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))==names-ignored
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('ok',str(total-ignored_count),'0',str(ignored_count),'0','0')]
    assert item['counts']==dict(passed=total-ignored_count,failed=0,ignored=ignored_count,measured=0,filtered_out=0)
for n,h in application['after_files'].items():assert sha(ROOT/n)==h
prior=read(ROOT/'驗證紀錄/current/native-model-generation-normal/validation-20261018-160708-195207/cohort.json');assert info['targets']['native_world_lighting_scene']['test_names']==prior['profiles'][profile]['targets']['native_world_lighting_scene']['test_names']
assert info['targets']['native_additional_finite_resources']['opted_in_counts']==dict(passed=1,failed=0,ignored=0,measured=0,filtered_out=0)
text=(run/profile/'finite-original-resources/stdout.log').read_text('utf-8-sig');assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('1','0','0','0','0')]
work=Path(info['resource_output']);assert work.resolve(strict=True).is_relative_to(ROOT/'LiTo381-Modern/target/current/native44FE-extra-finite-effects')
proof=Path(application['candidate_qualifications'][profile]['resources']);q=read(proof)
for n in ['original-effect-definitions.json','candidate-graphics.json']:assert sha(work/n)==q['raw_files']['run/'+n]
spec=read(work/'original-effect-definitions.json');graphics=read(work/'candidate-graphics.json');result=read(work/'resource-result.json');assert sha(work/'resource-result.json')==info['resource_result_sha256']
assert result['status']=='candidate_resource_render_and_source_event_progression_passed'and len(graphics)==11 and sorted(r['graphics']for r in result['cases'])==graphics
assert result['fixture_authority_unchanged_by_renderer']and not result['audio_device_playback_verified']and not result['gameplay_verified']
assert result['lifecycle']==dict(pre_cancel_preserves_old_model=True,same_map_new_generation_retires_bank=True,reused_sequence_fresh_sound_once=True,fresh_sound=116,late_native_callback_preserves_dwell=True)
sources={r['graphics']:r for r in spec['rows']};pictures={};total_sound=0
for row in result['cases']:
    gfx=row['graphics'];first=row['first_visible'];receipt=first['receipt'];bank=receipt['additional_bank'];final=row['final_receipt']
    assert first['pixels_changed']and bank['loaded']and bank['load_error']is None and receipt['load_error']is None
    assert bank['encoded_bytes']>0 and bank['retained_RGBA_bytes']>0 and bank['sources']and bank['sound_PCM_bytes']>0
    assert all(s['bytes']>0 and re.fullmatch('[0-9a-fA-F]{64}',s['sha256'])for s in bank['sources'])
    assert final['active']==0 and final['omitted']==0
    count=sum(len(f['events'])for f in sources[gfx]['action0']['frames']);assert row['source_sound_events']==count;total_sound+=count
    if gfx==2251:
        assert row['reviewed_blank_start_without_pixels']and row['source_sound_events']==1
        pinned=next(s for s in bank['sources']if s['name']=='2251-0.spr');assert pinned['bytes']==157993 and pinned['sha256'].upper()=='B90C450146296E77043090948B11075C0731FC7B15093734DC37935A114D975B'
    p=Path(first['png']).resolve(strict=True);assert p.parent.samefile(work)and p.name==f'effect-{gfx}.png'
    with Image.open(p)as im:assert im.size==(800,600)and im.mode=='RGBA';im.load()
    pictures[gfx]=dict(path=str(p),sha256=sha(p))
restoration=read(run/profile/'protected-outputs/restoration.json');assert restoration['existing_files_restored']
for n,h in restoration['before_files'].items():assert sha(ROOT/'LiTo381-Modern/target/current/verification'/n)==h
assert sha(run/profile/'protected-outputs/generated.zip')==restoration['generated_archive_sha256']
jobs=[p for p in c['process_history']if p['label'].startswith(profile+'/')];assert len(jobs)==19 and all(p['exit_code']==0 for p in jobs)
ids=','.join(str(p['pid'])for p in jobs);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/(profile+'-qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'reader.py').write_bytes(Path(__file__).read_bytes())
snapshot=out/'cohort-at-qualification.json';snapshot.write_bytes((run/'cohort.json').read_bytes())
files={f.relative_to(run).as_posix():f for job in jobs for f in(run/job['label']).iterdir()if f.is_file()};files['cohort.json']=snapshot;files['runtime/resource-result.json']=work/'resource-result.json';files.update({'images/'+Path(v['path']).name:Path(v['path'])for v in pictures.values()});pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
record=dict(status='independently_qualified_ability_finite_normal_related_scope',profile=profile,source_manifest_sha256=c['source_manifest_sha256'],source_files=1047,compiled_input_files=1048,normal=info['normal'],targets=info['targets'],passed=31,integration_passed=31,selected_library_passed=0,failed=0,default_ignored=2,live_probe_body_executed=False,opted_in_original_resource_tests=1,new_graphics_verified=11,source_sound_events=total_sound,original_PNGs=pictures,recorded_owned_processes=jobs,recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),source_overlays_used=False,normal_product_registered=False,full_library_verified=False,broad_default_verified=False,full_G06_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(out/'qualified.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=profile,passed=31,normal_sha256=info['normal']['sha256']),ensure_ascii=False))
