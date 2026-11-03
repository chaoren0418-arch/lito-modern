"""Qualify normal images and all 23 directly related scene cases, without borrowing CBC7 broad/library results."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,struct,subprocess,time,zipfile
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
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-model-generation-normal');c=read(run/'cohort.json');profile=a.profile;info=c['profiles'][profile];app=Path(c['application']);application=read(app/'application.json')
assert application['applied']and len(application['after_files'])==2 and sha(app/'application.json')==c['application_sha256']
assert c['source_manifest_sha256']==application['source_manifest_sha256']==sha(run/'source-inputs.json')
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1041 and len(read(run/'source-inputs.json'))==1040
for n,h in inputs.items():assert sha(ROOT/n)==h,n
ar=c['compiled_source_archive'];assert ar['files']==inputs and sha(ar['path'])==ar['sha256']
with zipfile.ZipFile(ar['path'])as z:
    assert set(z.namelist())==set(inputs)
    for n,h in inputs.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
assert sha(c['normal_entry'])==c['entry_sha256']and info['normal']['unique_entry_exit_code']==info['normal']['version_exit_code']==0
assert tuple(c['release_configuration'][k]for k in['opt-level','lto','strip','codegen-units'])==('z','thin','symbols',16);image(info['normal'])
assert set(info['targets'])=={'native_world_lighting_scene'};target=info['targets']['native_world_lighting_scene'];image(target);folder=run/profile/'native_world_lighting_scene';names=set(re.findall(r'^(.+): test$',(folder/'inventory/stdout.log').read_text('utf-8-sig'),re.M));assert names==set(target['test_names'])and len(names)==23
assert not re.findall(r'^(.+): test$',(folder/'ignored-inventory/stdout.log').read_text('utf-8-sig'),re.M)
text=(folder/'tests/stdout.log').read_text('utf-8-sig');assert set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))==names
assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('ok','23','0','0','0','0')]
assert target['counts']==dict(passed=23,failed=0,ignored=0,measured=0,filtered_out=0)
resource_key='LiTo381-Modern/tests/native_world_lighting_scene.rs'
with zipfile.ZipFile(app/'before.zip')as z:old=z.read(resource_key)
assert hashlib.sha256(old).hexdigest().upper()==application['before_files'][resource_key]
body=(ROOT/resource_key).read_bytes();assert body.startswith(old);prior_names=set(re.findall(r'#\[test\]\s*fn\s+(\w+)',old.decode('utf-8-sig')));added=set(re.findall(r'#\[test\]\s*fn\s+(\w+)',body[len(old):].decode('utf-8')));assert len(prior_names)==20 and len(added)==3 and not(prior_names&added)and names==prior_names|added and all(n.startswith('generation_model_')for n in added)
restoration=read(run/profile/'protected-outputs/restoration.json');assert restoration['existing_files_restored']
for n,h in restoration['before_files'].items():assert sha(ROOT/'LiTo381-Modern/target/current/verification'/n)==h
assert sha(run/profile/'protected-outputs/generated.zip')==restoration['generated_archive_sha256']
jobs=[p for p in c['process_history']if p['label'].startswith(profile+'/')];assert len(jobs)==6 and all(p['exit_code']==0 for p in jobs)
ids=','.join(str(p['pid'])for p in jobs);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress";raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[]
if isinstance(live,dict):live=[live]
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/(profile+'-qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();files={f.relative_to(run).as_posix():f for job in jobs for f in(run/job['label']).iterdir()if f.is_file()};snapshot=out/'cohort-at-qualification.json';snapshot.write_text(json.dumps(c,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');files['cohort.json']=snapshot;pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
record=dict(status='independently_qualified_model_generation_normal_and_scene23',profile=profile,source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,normal=info['normal'],targets=info['targets'],passed=23,integration_passed=23,selected_library_passed=0,failed=0,ignored=0,opted_in_original_resource_tests=0,prior_scene20_preserved_byte_prefix=True,new_generation_regressions=3,recorded_owned_processes=jobs,recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),full_library_verified=False,broad_default_verified=False,prior_CBC7_164_not_promoted=True,full_G06_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(out/'reader.py').write_bytes(Path(__file__).read_bytes());(out/'qualified.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=profile,passed=23,normal_sha256=info['normal']['sha256']),ensure_ascii=False))
