"""Independent readback after the original startup validation session exits."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,struct,subprocess,zipfile

ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def verify_archive(info):
    p=Path(info['path']).resolve(strict=True);assert p.is_relative_to(ROOT) and sha(p)==info['sha256']
    with zipfile.ZipFile(p) as z:
        assert len(z.namelist())==len(info['files']) and set(z.namelist())==set(info['files'])
        for name,h in info['files'].items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
def x64(path):
    with Path(path).open('rb') as f:
        assert f.read(2)==b'MZ';f.seek(0x3c);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset)
        assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
        f.seek(offset+24);assert struct.unpack('<H',f.read(2))[0]==0x20b

ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-startup') and a.terminal_exit_code==0
r=read(run/'cohort.json');assert r['status']=='executed_native_startup_scope_and_both_unique_entry_normal_builds' and r['owned_process'] is None
assert set(r['profiles'])=={'Debug','Release'} and all(p['exit_code']==0 or ('normal-config-invalid' in p['output'] or 'normal-config-relative' in p['output']) and p['exit_code']==2 for p in r['process_history'])
pins=read(run/'source-inputs.json');inputs=read(run/'compiled-inputs.json')
assert len(pins)==1024 and len(inputs)==1025 and inputs==pins|{'LiTo381-Modern/windows/app.manifest':sha(ROOT/'LiTo381-Modern/windows/app.manifest')}
assert sha(run/'source-inputs.json')==r['source_manifest_sha256']
for name,h in inputs.items():assert sha(ROOT/name)==h,name
assert sha(ROOT/'啟動現代客戶端.ps1')==sha(run/'entry.ps1')==r['entry_sha256']
assert r['compiled_source_archive']['files']==inputs;verify_archive(r['compiled_source_archive'])
profiles={}
for profile,info in r['profiles'].items():
    folder=run/profile;assert info['actual_test_executables']==4
    expected_jobs={'build':0,'normal-build':0,'normal-version':0,'normal-config-check':0,'normal-config-invalid':2,'normal-config-relative':2}
    for target in ['LiToModern','native_startup','native_resource_loading','native_login']:
        expected_jobs[target+'-list']=0;expected_jobs[target]=0
    for label,code in expected_jobs.items():
        jobs=[item for item in r['process_history'] if Path(item['output'])==folder/label]
        assert len(jobs)==1 and jobs[0]['exit_code']==code,(profile,label)
        assert Path(jobs[0]['arguments'][0]).is_absolute() or jobs[0]['arguments'][0]=='pwsh.exe'
    messages=[json.loads(line) for line in (folder/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
    assert any(item.get('reason')=='build-finished' and item['success'] for item in messages)
    artifacts=[item for item in messages if item.get('reason')=='compiler-artifact' and item.get('executable') and item['profile']['test']]
    assert len(artifacts)==4 and {item['target']['name'] for item in artifacts}==set(info['products'])
    for item in artifacts:
        product=info['products'][item['target']['name']]
        assert Path(item['executable']).resolve()==Path(product['path']).resolve()
        assert item['profile']==product['compiler_artifact_profile'] and item['features']==[]
    verify_archive(info['test_products_archive']);verify_archive(info['normal']['archive'])
    products=info['products'];assert set(products)=={'LiToModern','native_startup','native_resource_loading','native_login'}
    assert info['test_products_archive']['files']=={name+'.exe':v['sha256'] for name,v in products.items()}
    totals=dict(passed=0,failed=0,ignored=0);names={}
    for name,product in products.items():
        assert sha(product['path'])==product['sha256'];x64(product['path'])
        listed=(folder/(name+'-list')/'stdout.log').read_text(encoding='utf-8-sig')
        inventory=[s[:-6] for s in listed.splitlines() if s.endswith(': test')]
        assert len(inventory)==len(set(inventory)) and inventory
        actual=(folder/name/'stdout.log').read_text(encoding='utf-8-sig')
        rows=re.findall(r'^test (.+?) \.\.\. (ok|ignored(?:[^\r\n]*)?)$',actual,re.M)
        assert {name for name,_ in rows}==set(inventory) and len(rows)==len(inventory)
        m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',actual);assert m
        counts={key:int(value) for key,value in zip(['passed','failed','ignored','measured','filtered_out'],m.groups())}
        assert counts=={k:info['tests'][name][k] for k in counts} and counts['failed']==counts['measured']==counts['filtered_out']==0
        assert counts['passed']==sum(status=='ok' for _,status in rows) and counts['passed']+counts['ignored']==len(inventory)
        for key in totals:totals[key]+=counts[key]
        names[name]=inventory
    assert totals==info['counts'] and info['tests']['native_startup']['passed']==4
    normal=info['normal'];assert normal['unique_entry_exit_code']==0 and sha(normal['path'])==normal['sha256'];x64(normal['path'])
    assert normal['archive']['files']=={'LiToModern.exe':normal['sha256']}
    version=read(folder/'normal-version/stdout.log');assert version==info['version'] and version['build']==profile.lower() and version['pointer_bits']==64
    assert version['default_entry']=='native_login_setup' and version['native_settings_file']=='LiToModern.native.json'
    checked=read(folder/'normal-config-check/stdout.log');assert checked==info['config_check'] and checked['status']=='valid_native_settings'
    assert checked['login_endpoint']=='127.0.0.1:32000' and not checked['network_started'] and not checked['settings_written'] and not checked['resource_archives_verified']
    invalid=read(folder/'normal-config-invalid/stdout.log');relative=read(folder/'normal-config-relative/stdout.log')
    assert invalid['status']==relative['status']=='failed' and 'fixture-not-a-real-secret' not in json.dumps(invalid)
    profiles[profile]=dict(counts=totals,actual_test_executables=4,normal_sha256=normal['sha256'],normal_product_archive_sha256=normal['archive']['sha256'],test_products_archive_sha256=info['test_products_archive']['sha256'],test_names=names)

ids=','.join(str(p['pid']) for p in r['process_history'])
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live,dict):live=[live]
for v in live:
    for original in r['process_history']:
        if v['ProcessId']==original['pid']:assert abs((datetime.fromisoformat(v['created'])-datetime.fromisoformat(original['creation_utc'])).total_seconds())>0.001
out=run/'qualified';assert not out.exists()
files={'independent-qualifier.py':Path(__file__)}
for file in run.rglob('*'):
    if not file.is_file() or file.suffix=='.zip':continue
    assert file.resolve().is_relative_to(ROOT) and not file.is_symlink()
    files['run/'+file.relative_to(run).as_posix()]=file
hashes={name:sha(file) for name,file in files.items()};out.mkdir()
(out/'completion-CIM.json').write_bytes(p.stdout)
archive=out/'raw.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,file in files.items():z.write(file,name)
verify_archive(dict(path=str(archive),sha256=sha(archive),files=hashes))
q=dict(status='independently_qualified_native_startup_settings_and_normal_entry_scope',original_session_id=a.session_id,observed_terminal_exit_code=0,profiles=profiles,source_files=1024,compiled_input_files=1025,source_manifest_sha256=r['source_manifest_sha256'],compiled_source_archive_sha256=r['compiled_source_archive']['sha256'],entry_sha256=r['entry_sha256'],actual_formal_sessions=0,actual_owned_child_processes=len(r['process_history']),exact_owned_children_all_terminal=True,raw_archive_sha256=sha(archive),raw_files=hashes,GUI_verified=False,physical_input_verified=False,default_native_window_display_verified=False,full_current_regression_verified=False,old_source1021_results_relabelled=False,whole_goal_complete=False,qualifier_sha256=sha(Path(__file__)),qualified_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(qualification=str(out/'qualified.json'),profiles={k:v['counts'] for k,v in profiles.items()},GUI_verified=False),ensure_ascii=False))
