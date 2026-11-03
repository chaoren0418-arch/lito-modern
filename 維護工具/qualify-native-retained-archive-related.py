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
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-terrain-archive-retention')
c=read(run/'cohort.json');d=run/a.profile;info=c['profiles'][a.profile]
assert info['compile_exit_code']==0 and info['version_exit_code']==0
assert set(info['products'])==set(info['tests'])=={'LiToModern','native_resource_loading','native_terrain_residence','native_terrain_cache','native_arena_portal','native_startup','native_login'}
jobs=[p for p in c['process_history'] if Path(p['output']).is_relative_to(d)]
assert len(jobs)==10 and all(p['exit_code']==0 for p in jobs)
pins=read(run/'source-inputs.json');inputs=read(run/'compiled-inputs.json')
assert len(pins)==1024 and len(inputs)==1025 and all(inputs[n]==h for n,h in pins.items())
assert sha(run/'source-inputs.json')==c['source_manifest_sha256']
for name,h in inputs.items():assert sha(ROOT/name)==h,name
source=c['compiled_source_archive'];archive(source['path'],source['sha256'],inputs)
assert sha(ROOT/'啟動現代客戶端.ps1')==c['entry_sha256']
app=Path(c['application']);application=read(app/'application.json');before=read(app/'source-inputs-before.json')
assert [n for n in pins if pins[n]!=before[n]]==c['changed_sources'] and set(c['changed_sources'])==set(application['before'])
archive(app/'before-source.zip',application['before_archive_sha256'],application['before'])
red=read(app/'red/receipt.json');assert red['compile_exit_code']==0 and red['test_exit_code']==101
redpins=read(app/'red/source-inputs.json');redsource=app/'red/native_resource_loading.rs'
assert sha(redsource)==redpins['LiTo381-Modern/tests/native_resource_loading.rs']
assert function(redsource.read_text(encoding='utf-8-sig'),'observed_archive_lifetime_denies_writes_to_all_four_sources')==function((PROJECT/'tests/native_resource_loading.rs').read_text(encoding='utf-8-sig'),'observed_archive_lifetime_denies_writes_to_all_four_sources')
archive(app/'red/test-product.zip',red['test_archive_sha256'],{'native_resource_loading.exe':red['test_product_sha256']})
expected={'native_resource_loading':(7,0),'native_terrain_residence':(2,0),'native_terrain_cache':(3,0),
    'native_arena_portal':(2,2),'native_startup':(4,0),'native_login':(3,0),'LiToModern':(17 if a.profile=='Debug' else 13,0)}
messages=[json.loads(line) for line in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
artifacts={m['target']['name']:m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']}
assert set(artifacts)==set(expected)
names={};totals=dict(passed=0,failed=0,ignored=0)
for name,product in info['products'].items():
    exe=Path(product['path']).resolve(strict=True);assert exe.is_relative_to(PROJECT/'target/current')
    assert sha(exe)==product['sha256'] and Path(artifacts[name]['executable']).resolve()==exe
    assert artifacts[name]['features']==[] and artifacts[name]['profile']==product['compiler_profile'];amd64(exe)
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
assert names['native_resource_loading']['retained_archive_validation_and_failed_windows_keep_the_same_source_owner']=='ok'
assert totals==info['counts']
tests=info['test_products_archive'];archive(tests['path'],tests['sha256'],{name+'.exe':v['sha256'] for name,v in info['products'].items()})
normal=info['normal'];assert normal['unique_entry_exit_code']==0
assert sha(normal['path'])==normal['sha256'];amd64(normal['path'])
archive(normal['archive']['path'],normal['archive']['sha256'],{'LiToModern.exe':normal['sha256']})
assert not (d/'version/stderr.log').read_bytes().strip() and b'LiToModern' in (d/'version/stdout.log').read_bytes()
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert (release['opt-level'],release['lto'],release['codegen-units'],release['strip'])==('z','thin',16,'symbols')
terrain=read(d/'terrain-output/563-static.json');assert terrain['synthetic_authority_fixture']
assert terrain['first']['load_profile_us']['archive_sources_reused'] is False
assert terrain['before_boundary']['load_profile_us']['archive_sources_reused'] is True
assert terrain['after_boundary']['load_profile_us']['archive_sources_reused'] is True
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
for directory,prefix in [(d,a.profile+'/'),(app/'red','red/')]:
    for file in directory.rglob('*'):
        if file.is_file() and out not in file.parents and file.suffix.lower() not in ['.exe','.zip']:
            assert file.resolve().is_relative_to(directory) and not file.is_symlink()
            files[prefix+file.relative_to(directory).as_posix()]=file
raw=out/'raw.zip';hashes={name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(raw,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,p in files.items():z.write(p,name)
archive(raw,sha(raw),hashes)
result=dict(status='independently_qualified_retained_archive_related_profile',profile=a.profile,
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=c['source_manifest_sha256'],
    compiled_source_archive_sha256=source['sha256'],changed_sources=c['changed_sources'],
    actual_test_executables=7,counts=totals,all_test_results=names,test_products_archive_sha256=tests['sha256'],
    normal_sha256=normal['sha256'],normal_product_archive_sha256=normal['archive']['sha256'],
    red_test_original_exit_code=101,red_test_product_sha256=red['test_product_sha256'],same_regression_function_preserved=True,
    recorded_profile_owned_identities=len(jobs),recorded_profile_owned_processes_terminal=True,
    initial_archive_pins_us=terrain['first']['load_profile_us']['archive_pins_us'],
    reload_archive_pins_us={k:terrain[k]['load_profile_us']['archive_pins_us'] for k in ['before_boundary','after_boundary']},
    raw_archive_sha256=sha(raw),raw_files=hashes,actual_formal_sessions=0,new_siege_case_verified=False,
    GUI_verified=False,full_current_regression_verified=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({k:result[k] for k in ['status','profile','counts','normal_sha256','initial_archive_pins_us','reload_archive_pins_us']},ensure_ascii=False))
