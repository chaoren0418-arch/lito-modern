"""Compile source1009 background pixels and floor regression; no normal build."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess,zipfile

ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'floor-tests-20261016-102632-227125/normal-qualified'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
now=lambda:datetime.now(timezone.utc).isoformat()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
pins=read(NORMAL/'source-inputs.json');assert len(pins)==1005
pins['LiTo381-Modern/src/modern_lito/mod.rs']=sha(PROJECT/'src/modern_lito/mod.rs')
for n in ['src/modern_lito/native_background_pixels.rs','tests/native_background_pixels.rs','tests/fixtures/native-background-pixels-original.json','tests/fixtures/native-background-channels-original.bin']:
    pins['LiTo381-Modern/'+n]=sha(PROJECT/n)
assert len(pins)==1009
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
stable()
environment={k:v for k,v in os.environ.items() if (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS']) and v};assert environment=={}
command="$r='"+str(ROOT).replace("'","''")+"';$p=@(Get-CimInstance Win32_Process|Where-Object {$_.Name -in @('cargo.exe','rustc.exe') -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|native_|lito_modern-|oracle)' -or ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))});if($p.Count){exit 1}"
idle=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert idle.returncode==0,'Owned load remains active'
out=APP/('background-pixel-tests-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
save(out/'compiled-inputs.json',pins);(out/'runner.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'compiled-source.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n in pins:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'compiled-source.zip') as z:
    for n,h in pins.items():assert digest(z.read(n))==h,n
receipt={'status':'compiling_Debug_background_pixel_and_floor_tests','profile':'Debug','started_utc':now(),
 'compiled_inputs':1009,'suites':['native_background_pixels','native_terrain_pixels'],'results':{},'owned_process':None,
 'process_history':[],'normal_build_executed':False,'world_background_renderer_integrated':False,
 'full_default_library_TCP_regression_verified':False,'whole_goal_complete':False}
save(APP/'background-pixel-tests-current.json',{'path':str(out)})
persist=lambda:save(out/'receipt.json',receipt)
env=os.environ.copy();env.update(RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8');env['PATH']=str(RUST)+os.pathsep+env['PATH']
def run(command,folder):
    folder.mkdir()
    with (folder/'stdout.log').open('wb') as so,(folder/'stderr.log').open('wb') as se:
        c=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=so,stderr=se,creationflags=subprocess.CREATE_NO_WINDOW)
        identity={'pid':c.pid,'arguments':command,'observed_started_utc':now()};receipt['owned_process']=identity;receipt['process_history'].append(identity);persist()
        code=c.wait();identity.update(exit_code=code,finished_utc=now());receipt['owned_process']=None;persist()
    return code
def pe(b):
    e=int.from_bytes(b[60:64],'little');assert b[:2]==b'MZ' and b[e:e+4]==b'PE\0\0' and b[e+4:e+6]==b'\x64\x86' and b[e+24:e+26]==b'\x0b\x02'
persist()
try:
    cmd=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json']
    for n in receipt['suites']:cmd+=['--test',n]
    receipt['build_arguments']=cmd;persist();code=run(cmd,out/'build');receipt['compile_exit_code']=code;persist();assert code==0
    stable();products={};incidental={}
    for line in (out/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
        if not line.startswith('{'):continue
        v=json.loads(line)
        if v.get('reason')!='compiler-artifact' or not v.get('executable'):continue
        assert v['features']==[];path=Path(v['executable']).resolve(strict=True);assert path.is_relative_to(PROJECT/'target/current');pe(path.read_bytes())
        value={'path':str(path),'sha256':sha(path)}
        if v['profile']['test']:products[v['target']['name']]=value
        else:incidental[v['target']['name']]={**value,'normal_main_qualified':False}
    assert set(products)==set(receipt['suites']);save(out/'products.json',products);save(out/'incidental-bin-products.json',incidental)
    for name,items in [('test-products.zip',products),('incidental-bin-products.zip',incidental)]:
        with zipfile.ZipFile(out/name,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
            for n,p in items.items():z.write(p['path'],n+'.exe')
        with zipfile.ZipFile(out/name) as z:
            for n,p in items.items():assert digest(z.read(n+'.exe'))==p['sha256']
    for n,p in products.items():
        receipt.update(status='executing_Debug_background_pixel_and_floor_tests',active_suite=n);persist()
        code=run([p['path'],'--test-threads=1'],out/n);log=(out/n/'stdout.log').read_text(encoding='utf-8-sig')
        result=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log);assert len(result)==1
        status,passed,failed,ignored,measured,filtered=result[0]
        row=dict(exit_code=code,passed=int(passed),failed=int(failed),ignored=int(ignored),filtered=int(filtered),product_sha256=p['sha256']);receipt['results'][n]=row;persist()
        assert code==0 and status=='ok' and row['failed']==row['ignored']==row['filtered']==int(measured)==0
    stable();receipt.update(status='Debug_background_pixel_and_floor_tests_passed',passed=sum(v['passed'] for v in receipt['results'].values()),failed=0,ignored=0,source_inputs_unchanged=True,active_suite=None)
except BaseException as error:
    receipt.update(status='failed_preserved',error=repr(error));raise
finally:
    receipt['finished_utc']=now();persist()
print(json.dumps({'path':str(out),'status':receipt['status'],'passed':receipt.get('passed')},ensure_ascii=False))
