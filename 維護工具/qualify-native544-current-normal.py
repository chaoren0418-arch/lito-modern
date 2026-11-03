"""Independent source, PE, complete inventory and selected SPR result readback."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def archive(p,h,files):
    assert sha(p)==h
    with zipfile.ZipFile(p) as z:
        assert set(z.namelist())==set(files)
        for n,value in files.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',required=True,choices=['Debug','Release']);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native544-integration')
c=read(run/'cohort.json');profile=a.profile;info=c['profiles'][profile]
assert info['sprite_selection']==dict(passed=18,failed=0,ignored=1,measured=0,filtered_out=2392)
assert info['original_resource_optin']==dict(passed=1,failed=0,ignored=0,measured=0,filtered_out=2410)
assert info['registered_total']==2411 and info['unchanged_registered_ignored']==473
jobs=[j for j in c['process_history'] if j['label'].startswith(profile+'/')];assert len(jobs)==7 and all(j['exit_code']==0 for j in jobs)
assert {j['label'].split('/')[1] for j in jobs}=={'normal-build','version','lib-build','inventory','ignored-inventory','sprite-tests','sprite-resource'}
inputs=read(run/'compiled-inputs.json');pins=read(run/'source-inputs.json');assert len(inputs)==1029 and len(pins)==1028
assert sha(run/'source-inputs.json')==c['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
source=c['compiled_source_archive'];archive(source['path'],source['sha256'],source['files']);assert source['files']==inputs
application=Path(c['application']);app=read(application/'application.json')
assert sha(application/'application.json')==c['application_sha256'] and app['source_manifest_sha256']==c['source_manifest_sha256']
assert app['changed_sources']==['LiTo381-Modern/src/modern_lito/sprite.rs']
assert app['after']['LiTo381-Modern/src/modern_lito/sprite.rs']==inputs['LiTo381-Modern/src/modern_lito/sprite.rs']
component=Path(app['candidate_qualification']);q=read(component)
assert sha(component)==app['candidate_qualification_sha256'] and q['candidate_source_sha256']==inputs['LiTo381-Modern/src/modern_lito/sprite.rs']
assert q['flat_pixel_matches']==q['compact_pixel_matches']==31 and q['normalized_empty_frames']==32
for kind,member in [('normal','LiToModern.exe'),('test','lito_modern.exe')]:
    p=Path(info[kind]['path']);assert sha(p)==info[kind]['sha256'];z=info[kind]['archive'];archive(z['path'],z['sha256'],{member:info[kind]['sha256']})
    with p.open('rb') as f:
        head=f.read(64);assert head[:2]==b'MZ';f.seek(int.from_bytes(head[60:64],'little'));pe=f.read(26)
        assert pe[:4]==b'PE\0\0' and pe[4:6]==b'\x64\x86' and pe[24:26]==b'\x0b\x02'
assert info['normal']['unique_entry_exit_code']==info['normal']['version_exit_code']==0
entry=next(j for j in jobs if j['label'].endswith('/normal-build'))
assert entry['command']==['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile]
assert sha(ROOT/'啟動現代客戶端.ps1')==c['entry_sha256']
build=next(j for j in jobs if j['label'].endswith('/lib-build'))
expected=['test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--lib']+(['--release'] if profile=='Release' else [])
assert build['command'][1:]==expected
def text(label):return (run/profile/label/'stdout.log').read_text(encoding='utf-8-sig')
def names(label):return set(re.findall(r'^(.+): test$',text(label),re.M))
old=read(Path(c['original_inventory_cohort'])/profile/'inventory-audit/inventory.json')
new='modern_lito::sprite::tests::reviewed544_marker_preserves_slots_and_rejects_corruption'
assert names('inventory')==set(old['all_test_names'])|{new} and names('ignored-inventory')==set(old['ignored_test_names'])
rows=dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',text('sprite-tests'),re.M))
assert len(rows)==19 and rows[new]=='ok' and {n for n,s in rows.items() if s=='ignored'}=={'modern_lito::sprite::tests::real_shared_spr_rgba_matches_csharp_reference'}
assert all(s=='ok' for n,s in rows.items() if s!='ignored')
for label,counts in [('sprite-tests',('18','0','1','0','2392')),('sprite-resource',('1','0','0','0','2410'))]:
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text(label))==[('ok',*counts)]
assert re.findall(r'^test (.+?) \.\.\. ok$',text('sprite-resource'),re.M)==['modern_lito::sprite::tests::real_shared_spr_rgba_matches_csharp_reference']
ids=','.join(str(j['pid']) for j in jobs)
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for old in jobs:
        if row['ProcessId']==old['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(old['creation_utc'])).total_seconds())>.001
out=run/profile/'qualified';out.mkdir();(out/'CIM.json').write_bytes(raw)
files={'cohort.json':run/'cohort.json','application.json':application/'application.json','candidate-component-qualified.json':component,'CIM.json':out/'CIM.json','qualifier.py':Path(__file__)}
for job in jobs:
    for n in ['stdout.log','stderr.log']:files[job['label']+'/'+n]=run/job['label']/n
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
archive(out/'raw.zip',sha(out/'raw.zip'),hashes)
q=dict(status='independently_qualified_current_exact544_normal_and_sprite_profile',profile=profile,source_manifest_sha256=c['source_manifest_sha256'],source_files=1028,compiled_input_files=1029,
    normal_sha256=info['normal']['sha256'],test_sha256=info['test']['sha256'],normal_path=info['normal']['path'],normal_archive_sha256=info['normal']['archive']['sha256'],test_archive_sha256=info['test']['archive']['sha256'],
    registered_library_tests=2411,unchanged_registered_ignored=473,default_sprite_selection=info['sprite_selection'],explicit_original_resource=info['original_resource_optin'],unique_sprite_tests_executed=19,
    recorded_profile_jobs=7,recorded_profile_jobs_all_terminal=True,referenced_exact_source_component=str(component),referenced_component_sha256=sha(component),component_63_frame_comparison_reused_by_exact_source=True,
    full_library_test_bodies_executed=False,actual_formal_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,raw_files=hashes,raw_archive_sha256=sha(out/'raw.zip'))
save(out/'qualified.json',q)
print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=profile,normal_sha256=q['normal_sha256'],sprite_passed=18,original_resource_passed=1),ensure_ascii=False))
