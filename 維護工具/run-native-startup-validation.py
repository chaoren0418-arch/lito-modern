"""Bounded startup/default-entry regression and unique-entry normal builds."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,zipfile

ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
now=lambda:datetime.now(timezone.utc).isoformat()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def archive(path,files):
    pins={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(pins) and set(z.namelist())==set(pins)
        for n,h in pins.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    return dict(path=str(path),sha256=sha(path),files=pins,readback_verified=True)

ap=argparse.ArgumentParser();ap.add_argument('application',type=Path);a=ap.parse_args()
application=a.application.resolve(strict=True);assert application.is_relative_to(ROOT/'驗證紀錄/current/native-startup')
app=read(application/'application.json');assert app['source_applied']
assert read(app['previous_default_completion'])['observed_terminal_exit_code']==0
pins=read(application/'source-inputs.json');assert len(pins)==1024
inputs=pins|{'LiTo381-Modern/windows/app.manifest':sha(PROJECT/'windows/app.manifest')}
assert len(inputs)==1025
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
out=ROOT/'驗證紀錄/current/native-startup'/('validation-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
save(out/'source-inputs.json',pins);save(out/'compiled-inputs.json',inputs)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
entry=ROOT/'啟動現代客戶端.ps1';entry_hash=sha(entry);(out/'entry.ps1').write_bytes(entry.read_bytes())
source_archive=archive(out/'compiled-source.zip',{n:ROOT/n for n in inputs})
state=dict(status='prepared_native_startup_validation',application=str(application),output=str(out),source_files=1024,compiled_inputs=1025,source_manifest_sha256=sha(out/'source-inputs.json'),compiled_source_archive=source_archive,entry_sha256=entry_hash,profiles={},process_history=[],owned_process=None,GUI_verified=False,physical_input_verified=False,whole_goal_complete=False,started_utc=now())
persist=lambda:save(out/'cohort.json',state)
env=os.environ.copy();env.update(RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
env['PATH']=str(RUST)+os.pathsep+env['PATH']
for key in list(env):
    if key.startswith('LITO_'):del env[key]
def check():
    for name,h in inputs.items():assert sha(ROOT/name)==h,name
    assert sha(entry)==entry_hash
    assert sha(Path(__file__))==sha(out/'runner.py')
def execute(command,folder,cwd=PROJECT):
    folder.mkdir(parents=True,exist_ok=False);check()
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=cwd,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];get=ctypes.WinDLL('kernel32',use_last_error=True).GetProcessTimes
        get.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;get.restype=wintypes.BOOL
        assert get(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(v) for v in times])
        ticks=(times[0].dwHighDateTime<<32)|times[0].dwLowDateTime
        identity=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=command,output=str(folder),exit_code=None)
        state['owned_process']=identity;state['process_history'].append(identity);persist()
        while True:
            try:code=child.wait(timeout=30);break
            except subprocess.TimeoutExpired:
                persist();print(json.dumps(dict(status='original_owned_child_live',pid=child.pid,profile=state.get('active_profile'),phase=state['status'])),flush=True)
    identity['exit_code']=code;identity['finished_utc']=now();state['owned_process']=None;persist();check();return code
def summary(p):
    text=p.read_text(encoding='utf-8-sig')
    m=re.search(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',text)
    assert m,text[-1000:]
    result=dict(status=m[1],**{k:int(v) for k,v in zip(['passed','failed','ignored','measured','filtered_out'],m.groups()[1:])})
    assert result['status']=='ok' and result['failed']==result['measured']==result['filtered_out']==0
    return result

persist();print('NATIVE_STARTUP_VALIDATION='+str(out),flush=True)
try:
    for profile in ['Debug','Release']:
        d=out/profile;d.mkdir();state['active_profile']=profile;state['status']='compiling_native_startup_scope';state['profiles'][profile]={};persist()
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--bin','LiToModern','--test','native_startup','--test','native_resource_loading','--test','native_login']
        if profile=='Release':command.append('--release')
        assert execute(command,d/'build')==0,'Compiler failure preserved.'
        messages=[json.loads(l) for l in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
        artifacts={v['target']['name']:v for v in messages if v.get('reason')=='compiler-artifact' and v.get('executable') and v['profile']['test']}
        assert set(artifacts)=={'LiToModern','native_startup','native_resource_loading','native_login'}
        products={};tests={}
        state['status']='running_native_startup_scope';persist()
        for name,artifact in artifacts.items():
            exe=Path(artifact['executable']).resolve(strict=True);assert exe.is_relative_to(PROJECT/'target/current') and artifact['features']==[]
            products[name]=dict(path=str(exe),sha256=sha(exe),compiler_artifact_profile=artifact['profile'])
            assert execute([str(exe),'--list','--format','terse'],d/(name+'-list'))==0
            assert execute([str(exe),'--test-threads=1'],d/name)==0,'Test failure preserved.'
            tests[name]=summary(d/name/'stdout.log');state['profiles'][profile]['tests']=tests;persist()
        product_archive=archive(d/'test-products.zip',{n+'.exe':Path(v['path']) for n,v in products.items()})
        state['profiles'][profile].update(products=products,test_products_archive=product_archive)
        state['status']='building_unique_entry_normal';persist()
        assert execute(['pwsh.exe','-NoProfile','-File',str(entry),'-BuildOnly','-BuildProfile',profile],d/'normal-build')==0
        normal=PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe'
        normal_archive=archive(d/'normal-product.zip',{'LiToModern.exe':normal})
        state['profiles'][profile]['normal']=dict(path=str(normal),sha256=sha(normal),unique_entry_exit_code=0,archive=normal_archive)
        state['status']='checking_noninteractive_normal_startup_routes';persist()
        assert execute([str(normal),'--version'],d/'normal-version',cwd=out)==0
        version=read(d/'normal-version/stdout.log');assert version['default_entry']=='native_login_setup' and version['native_settings_file']=='LiToModern.native.json'
        assert not version['launches_native_bin'] and not version['launches_tw'] and version['pointer_bits']==64
        fixture=d/'native-settings.json';save(fixture,dict(schema=1,login_endpoint='127.0.0.1:32000',resource_root=r'D:\lineage開發\0-客戶端'))
        assert execute([str(normal),'--check-native-config',str(fixture)],d/'normal-config-check',cwd=out)==0
        config=read(d/'normal-config-check/stdout.log');assert config['status']=='valid_native_settings' and not config['network_started'] and not config['settings_written'] and not config['resource_archives_verified']
        invalid=d/'invalid-native-settings.json';save(invalid,read(fixture)|{'password':'fixture-not-a-real-secret'})
        assert execute([str(normal),'--check-native-config',str(invalid)],d/'normal-config-invalid',cwd=out)==2
        failure=read(d/'normal-config-invalid/stdout.log');assert failure['status']=='failed'
        assert 'fixture-not-a-real-secret' not in (d/'normal-config-invalid/stdout.log').read_text(encoding='utf-8-sig')
        assert execute([str(normal),'--check-native-config','relative.json'],d/'normal-config-relative',cwd=out)==2
        state['profiles'][profile].update(version=version,config_check=config,actual_test_executables=4,counts={k:sum(v[k] for v in tests.values()) for k in ['passed','failed','ignored']});persist()
        print(json.dumps(dict(profile=profile,counts=state['profiles'][profile]['counts'],normal_sha256=sha(normal)),ensure_ascii=False),flush=True)
    state.update(status='executed_native_startup_scope_and_both_unique_entry_normal_builds',active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    state['finished_utc']=now();persist()
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
