"""Build and exercise the retained geometry correction, then the unique normals."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,zipfile

ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
now=lambda:datetime.now(timezone.utc).isoformat()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def retain(path,files):
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,h in hashes.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    return dict(path=str(path),sha256=sha(path),files=hashes,readback_verified=True)

application=ROOT/'驗證紀錄/current/native-retained-geometry/application-20261017-003701-995961'
app=read(application/'application.json')
red=read(application/'red-failure-readback.json')
assert red['test_exit_code']==101 and red['observed_terminal_exit_code']==1
assert sha(application/'red-failure.zip')==red['archive_sha256']
before=read(application/'base-source-inputs.json')
pins={name:sha(ROOT/name) for name in before}
changed=[name for name in pins if pins[name]!=before[name]]
assert len(pins)==1024 and set(changed)==set(app['before'])
inputs=pins|{'LiTo381-Modern/windows/app.manifest':sha(PROJECT/'windows/app.manifest')}
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
targets=['native_resource_loading','native_terrain_residence','native_terrain_cache','native_arena_portal','native_startup','native_login','native_navigation','native_world_layers','native_observer']
out=ROOT/'驗證紀錄/current/native-retained-geometry'/('validation-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
save(out/'source-inputs.json',pins);save(out/'compiled-inputs.json',inputs)
source=retain(out/'compiled-source.zip',{name:ROOT/name for name in inputs})
entry_hash=sha(ROOT/'啟動現代客戶端.ps1')
state=dict(status='prepared_native_retained_geometry_related_validation',output=str(out),application=str(application),
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=sha(out/'source-inputs.json'),
    compiled_source_archive=source,changed_sources=changed,entry_sha256=entry_hash,profiles={},
    owned_process=None,process_history=[],actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False,started_utc=now())
persist=lambda:save(out/'cohort.json',state)
env=os.environ.copy();env.update(RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),
    PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
env['PATH']=str(RUST)+os.pathsep+env['PATH']
def check():
    for name,h in inputs.items():assert sha(ROOT/name)==h,name
    assert sha(ROOT/'啟動現代客戶端.ps1')==entry_hash
def execute(command,folder):
    folder.mkdir()
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)]
        get=ctypes.WinDLL('kernel32',use_last_error=True).GetProcessTimes
        get.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;get.restype=wintypes.BOOL
        assert get(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(v) for v in times])
        ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        item=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=command,output=str(folder))
        state['owned_process']=item;state['process_history'].append(item);persist()
        code=p.wait();item.update(exit_code=code,finished_utc=now());state['owned_process']=None;persist()
    return code

persist();print(json.dumps(dict(output=str(out),source_manifest_sha256=state['source_manifest_sha256']),ensure_ascii=False),flush=True)
try:
    for profile in ['Debug','Release']:
        check();d=out/profile;d.mkdir();state.update(status='compiling_retained_geometry_related_targets',active_profile=profile)
        info=dict(tests={},products={});state['profiles'][profile]=info;persist()
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--bin','LiToModern']
        if profile=='Release':command.append('--release')
        for target in targets:command+=['--test',target]
        code=execute(command,d/'build');info['compile_exit_code']=code;persist();assert code==0
        messages=[json.loads(l) for l in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
        artifacts=[m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']]
        assert {m['target']['name'] for m in artifacts}==set(targets)|{'LiToModern'} and len(artifacts)==len(targets)+1
        for m in artifacts:
            assert m['features']==[];p=Path(m['executable']).resolve(strict=True);assert p.is_relative_to(PROJECT/'target/current')
            info['products'][m['target']['name']]=dict(path=str(p),sha256=sha(p),compiler_profile=m['profile'])
        info['test_products_archive']=retain(d/'test-products.zip',{name+'.exe':Path(v['path']) for name,v in info['products'].items()});persist()
        state['status']='running_retained_geometry_related_targets';persist()
        for name,product in info['products'].items():
            code=execute([product['path'],'--test-threads=1'],d/('test-'+name))
            log=(d/('test-'+name)/'stdout.log').read_text(encoding='utf-8-sig')
            matches=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
            assert len(matches)==1
            status,passed,failed,ignored,measured,filtered=matches[0]
            info['tests'][name]=dict(exit_code=code,status=status,passed=int(passed),failed=int(failed),ignored=int(ignored),measured=int(measured),filtered=int(filtered));persist()
            assert code==0 and status=='ok' and int(failed)==int(measured)==int(filtered)==0
        render_root=PROJECT/'target/current/verification/native-terrain-residence'
        terrain=d/'terrain-output';terrain.mkdir()
        for name in ['563-static.json','563-before.png','563-after.png']:
            (terrain/name).write_bytes((render_root/name).read_bytes())
        info['actual_test_executables']=len(targets)+1
        info['counts']={key:sum(v[key] for v in info['tests'].values()) for key in ['passed','failed','ignored']};persist()
        check();state['status']='building_unique_entry_normal';persist()
        code=execute(['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile],d/'normal-build')
        assert code==0
        (d/'entry-build.log').write_bytes((ROOT/'驗證紀錄/current'/('build-release.log' if profile=='Release' else 'build.log')).read_bytes())
        exe=PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe'
        info['normal']=dict(path=str(exe),sha256=sha(exe),unique_entry_exit_code=0,
            archive=retain(d/'normal-product.zip',{'LiToModern.exe':exe}));persist()
        assert execute([str(exe),'--version'],d/'version')==0
        info['version_exit_code']=0;persist();check()
        print(json.dumps(dict(profile=profile,counts=info['counts'],normal=info['normal']['sha256']),ensure_ascii=False),flush=True)
    state.update(status='executed_retained_geometry_related_tests_and_unique_normals',active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    state['finished_utc']=now();persist()
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
