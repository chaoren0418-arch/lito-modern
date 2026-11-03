"""Same-source scene/publication regressions, then normal unique-entry builds."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import ctypes, hashlib, json, os, re, subprocess, tomllib, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
BASE = ROOT / '驗證紀錄/current/native-actor-green-clock'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
SCOPES = {'Debug': BASE / 'scoped-20261016-160106-048570-Debug',
          'Release': BASE / 'scoped-20261016-160403-484605-Release'}
SUITES = ['native_world_lighting_scene', 'native_world', 'native_motion',
          'native_world_layers', 'native_terrain_residence', 'native_light_lifecycle']
GROUPS = {'lib-world-raster': 'world_raster::tests::', 'lib-world-scene': 'world_scene::tests::'}
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
now = lambda: datetime.now(timezone.utc).isoformat()
def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')
def zipped_entry(z, name):
    with z.open(name) as f: return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def archive(path, files):
    hashes = {name: sha(p) for name, p in files.items()}
    with zipfile.ZipFile(path, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        for name, p in files.items(): z.write(p, name)
    with zipfile.ZipFile(path) as z:
        assert set(z.namelist()) == set(hashes)
        for name, h in hashes.items(): assert zipped_entry(z, name) == h
    return dict(sha256=sha(path), files=hashes)
def check_pe(path):
    with Path(path).open('rb') as f:
        h=f.read(64); assert h[:2]==b'MZ'; f.seek(int.from_bytes(h[60:64], 'little')); pe=f.read(26)
    assert pe[:4]==b'PE\0\0' and pe[4:6]==b'\x64\x86' and pe[24:26]==b'\x0b\x02'

pins = read(SCOPES['Debug'] / 'source-inputs.json')
assert len(pins)==1021 and read(SCOPES['Release'] / 'source-inputs.json')==pins
for profile, p in SCOPES.items():
    q=read(p/'qualified.json')
    assert q['status']=='independently_qualified_source1021_green_shared_clock_scope'
    assert q['profile']==profile and (q['passed'],q['failed'],q['ignored'])==(36,0,0)
    assert q['source_manifest_sha256']=='180741A87501C13178B8521318365810B458324663F9D419758AF983099415E9'
def check():
    for name, h in pins.items(): assert sha(ROOT/name)==h, ('source drift', name)
    actual={str(p.relative_to(ROOT)).replace('\\','/') for folder in ['src','tests']
            for p in (PROJECT/folder).rglob('*') if p.is_file()}
    declared={n for n in pins if n.startswith(('LiTo381-Modern/src/','LiTo381-Modern/tests/'))}
    assert actual==declared, ('undeclared or missing src/tests', sorted(actual^declared))
check()
overrides={k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
assert not overrides, overrides
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert release['opt-level']=='z' and release['lto']=='thin' and release['codegen-units']==16 and release['strip']=='symbols'
root_literal=str(ROOT).replace("'", "''")
idle_script="""[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot='ROOT_LITERAL'
@(Get-CimInstance Win32_Process|Where-Object {
 $_.Name -in @('cargo.exe','rustc.exe') -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($taskRoot)) -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -and
 ($_.Name -match '^(LiToModern|native_|lito_modern-)' -or ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))
}|Select-Object ProcessId,Name,CreationDate,CommandLine)|ConvertTo-Json -Depth 4 -Compress
""".replace('ROOT_LITERAL',root_literal)
idle=subprocess.run(['powershell.exe','-NoProfile','-Command',idle_script],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert idle.returncode==0 and not idle.stderr
assert not idle.stdout.strip() or json.loads(idle.stdout.decode('utf-8-sig'))==[], 'Wait on original live workload handle.'

out=BASE/('related-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True,exist_ok=False)
save(out/'source-inputs.json',pins)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
(out/'entry-script-before.ps1').write_bytes((ROOT/'啟動現代客戶端.ps1').read_bytes())
save(out/'idle-before.json',dict(observed_utc=now(),processes=[],raw_stdout_hex=idle.stdout.hex()))
receipt=dict(status='prepared',output=str(out),source_files=1021,features=[],suites=SUITES,library_filters=GROUPS,
    previous_scope_qualifications={k:dict(path=str(v/'qualified.json'),sha256=sha(v/'qualified.json')) for k,v in SCOPES.items()},
    source_manifest_sha256=sha(out/'source-inputs.json'),entry_script_sha256=sha(ROOT/'啟動現代客戶端.ps1'),
    profile_environment=overrides,profiles={},normal_products={},owned_process=None,process_history=[],
    full_default_regression_verified=False,full_library_regression_verified=False,full_current_TCP_regression_verified=False,
    current_GUI_qualified=False,whole_goal_complete=False,started_utc=now())
receipt['source_archive']=archive(out/'compiled-source.zip',{n:ROOT/n for n in pins})
persist=lambda:save(out/'receipt.json',receipt)
persist(); print(json.dumps(dict(output=str(out),status=receipt['status']),ensure_ascii=False),flush=True)
env=os.environ.copy()
env.update(RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),TEMP=str(PROJECT/'target/current/scratch'),
           TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
env['PATH']=str(RUST)+os.pathsep+env['PATH']
def execute(command, folder):
    folder.mkdir(parents=True,exist_ok=False)
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)]
        get=ctypes.WinDLL('kernel32',use_last_error=True).GetProcessTimes
        get.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;get.restype=wintypes.BOOL
        assert get(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(v) for v in times])
        ticks=(times[0].dwHighDateTime<<32)|times[0].dwLowDateTime
        created=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
        item=dict(pid=child.pid,creation_utc=created.isoformat(),args=command,log_directory=str(folder))
        receipt['process_history'].append(item);receipt['owned_process']=item;persist()
        code=child.wait(); item.update(exit_code=code,finished_utc=now());receipt['owned_process']=None;persist()
    return code
try:
    for profile in ['Debug','Release']:
        check();d=out/profile;d.mkdir()
        r=dict(compile_exit_code=None,results={});receipt['profiles'][profile]=r
        receipt.update(status='compiling_related_scope',active_profile=profile);persist()
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc',
                 '--target-dir','target/current','--message-format=json','--lib']
        if profile=='Release': command.append('--release')
        for name in SUITES: command+=['--test',name]
        r['build_arguments']=command;r['compile_exit_code']=execute(command,d/'build');persist()
        assert r['compile_exit_code']==0, 'Preserve compilation failure.'
        check();products={}
        messages=[json.loads(x) for x in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if x.startswith('{')]
        assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
        for m in messages:
            if m.get('reason')!='compiler-artifact' or not m.get('executable') or not m['profile']['test']: continue
            name=m['target']['name'];assert name in set(SUITES)|{'lito_modern'} and name not in products and m['features']==[]
            p=Path(m['executable']).resolve(strict=True);assert p.is_relative_to(PROJECT/'target/current');check_pe(p)
            products[name]=dict(path=str(p),sha256=sha(p),machine='AMD64',optional_header='PE32+')
        assert set(products)==set(SUITES)|{'lito_modern'}
        save(d/'products.json',products)
        r['test_archive']=archive(d/'test-products.zip',{n+'.exe':Path(p['path']) for n,p in products.items()})
        cases=[(n,n,[]) for n in SUITES]+[(n,'lito_modern',[f]) for n,f in GROUPS.items()]
        for label,name,filters in cases:
            p=products[name];assert sha(p['path'])==p['sha256']
            receipt.update(status='running_related_scope',active_target=label);persist()
            inventory=execute([p['path'],*filters,'--list'],d/(label+'-inventory'));assert inventory==0
            names=re.findall(r'^(.+): test$',(d/(label+'-inventory')/'stdout.log').read_text(encoding='utf-8-sig'),re.M)
            assert names and len(names)==len(set(names))
            code=execute([p['path'],*filters,'--test-threads=1'],d/(label+'-runtime'))
            text=(d/(label+'-runtime')/'stdout.log').read_text(encoding='utf-8-sig')
            matches=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
            assert len(matches)==1
            state,passed,failed,ignored,measured,filtered=matches[0]
            passed,failed,ignored,measured,filtered=map(int,[passed,failed,ignored,measured,filtered])
            assert passed+failed+ignored==len(names) and measured==0 and (filters or filtered==0)
            r['results'][label]=dict(product=name,product_sha256=p['sha256'],filters=filters,inventory=names,exit_code=code,
                passed=passed,failed=failed,ignored=ignored,filtered=filtered)
            persist();print(f'{profile} {label}: {passed} passed/{failed} failed/{ignored} ignored',flush=True)
            assert state=='ok' and code==failed==0, 'Preserve runtime failure.'
        r.update({key:sum(v[key] for v in r['results'].values()) for key in ['passed','failed','ignored']})
        r.update(actual_test_executables=len(products),runtime_groups=len(cases));check();persist()
    for profile in ['Debug','Release']:
        check();receipt.update(status='building_unique_entry_normal',active_profile=profile,active_target=None);persist()
        command=['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile]
        folder=out/('normal-'+profile);code=execute(command,folder);assert code==0
        p=PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe';check_pe(p)
        log=ROOT/'驗證紀錄/current'/('build-release.log' if profile=='Release' else 'build.log')
        (folder/'entry-build.log').write_bytes(log.read_bytes())
        product=dict(path=str(p),sha256=sha(p),unique_entry_build_exit_code=code,build_arguments=command,features=[],machine='AMD64',optional_header='PE32+')
        product['archive']=archive(folder/'product.zip',{'LiToModern.exe':p})
        receipt['normal_products'][profile]=product;persist()
    check();assert sha(ROOT/'啟動現代客戶端.ps1')==receipt['entry_script_sha256']
    receipt.update(status='executed_green_clock_related_scope_and_unique_entry_normal_builds',sources_unchanged=True,
                   active_profile=None,active_target=None)
except BaseException as error:
    receipt.update(status='failed_preserved',error=repr(error));raise
finally:
    receipt['finished_utc']=now();persist()
    files={str(p.relative_to(out)).replace('\\','/'):p for p in out.rglob('*') if p.is_file() and p.suffix.lower()!='.zip'}
    save(out/'archive-readback.json',archive(out/'raw.zip',files))
print(json.dumps(dict(status=receipt['status'],output=str(out),counts={k:{f:v[f] for f in ['passed','failed','ignored']} for k,v in receipt['profiles'].items()}),ensure_ascii=False),flush=True)
