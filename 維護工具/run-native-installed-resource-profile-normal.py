"""Build both normal products and run the affected ordinary integration targets."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,sys,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current';RUST=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('application',type=Path);a=ap.parse_args();app=a.application.resolve(strict=True);assert app.is_relative_to(V/'native-installed-resource-profile-normal')
application=read(app/'application.json');source=application['source_manifest_sha256'];inputs=read(app/'applied-compiled-inputs.json');assert len(inputs)==1049 and sha(app/'applied-source-inputs.json')==source
queue=read(ROOT/'工作佇列.json');assert queue['current_product_identity']['source_manifest_sha256']==source and not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
assert not {key:value for key,value in os.environ.items()if value and(key.startswith('CARGO_PROFILE_')or key in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
release=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['profile']['release'];assert tuple(release[x]for x in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    rows=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(row)for row in rows]);ticks=rows[0].dwLowDateTime|(rows[0].dwHighDateTime<<32);return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
out=V/'native-installed-resource-profile-normal'/('validation-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'runner.py').write_bytes(Path(__file__).read_bytes())
for name in ['source-inputs','compiled-inputs']:(out/(name+'.json')).write_bytes((app/('applied-'+name+'.json')).read_bytes())
state=dict(status='prepared_installed_resource_profile_normal_related',application=str(app),source_manifest_sha256=source,source_files=1048,compiled_input_files=1049,runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),profiles={},process_history=[],owned_process=None,source_overlays_used=False,diagnostic_API_extension=False,normal_product_build=True,actual_sessions=0,GUI_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state);print('CATALOG_NORMAL_VALIDATION='+str(out),flush=True)
env={key:value for key,value in os.environ.items()if not key.startswith('LITO_')};env.update(PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'));env['PATH']=str(RUST)+os.pathsep+env['PATH']
def guard():
    for name,digest in inputs.items():assert sha(ROOT/name)==digest,name
def execute(command,folder):
    guard();folder.mkdir()
    with(folder/'stdout.log').open('wb')as stdout,(folder/'stderr.log').open('wb')as stderr:
        child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,output=str(folder));state['process_history'].append(row);state['owned_process']=row;save(out/'cohort.json',state)
        try:row['exit_code']=child.wait()
        except BaseException:
            if child.poll()is None:
                assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc'];row['cleanup_stopped_original_owned_handle']=True;child.terminate();child.wait(timeout=3)
            row['exit_code']=child.returncode;raise
        finally:state['owned_process']=None;save(out/'cohort.json',state)
    guard();assert row['exit_code']==0,'preserve original normal build/test failure';return row
try:
    for profile in ['Debug','Release']:
        d=out/profile;d.mkdir();info={};state['profiles'][profile]=info;state.update(status='building_installed_resource_profile_normal',active_profile=profile);save(out/'cohort.json',state)
        execute(['pwsh','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile],d/'normal-build')
        normal=P/('target/current/x86_64-pc-windows-msvc/'+profile.lower()+'/LiToModern.exe');info['normal']=dict(path=str(normal),sha256=sha(normal));save(out/'cohort.json',state)
        state['status']='compiling_installed_resource_profile_related';save(out/'cohort.json',state)
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--test','native_teleport','--test','native_services','--test','native_login']
        if profile=='Release':command.append('--release')
        execute(command,d/'test-build');artifacts=[]
        for line in(d/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
            if not line.startswith('{'):continue
            row=json.loads(line)
            if row.get('reason')=='compiler-artifact'and row.get('executable')and row['profile']['test']:artifacts.append(row)
        assert {row['target']['name']for row in artifacts}=={'native_teleport','native_services','native_login'}and len(artifacts)==3
        info['targets']={};state['status']='running_installed_resource_profile_related';save(out/'cohort.json',state)
        for artifact in artifacts:
            name=artifact['target']['name'];exe=Path(artifact['executable']).resolve(strict=True);assert exe.is_relative_to(P/'target/current')and artifact['features']==[]
            execute([str(exe),'--test-threads=1'],d/name);text=(d/name/'stdout.log').read_text('utf-8-sig');result=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text);assert len(result)==1
            passed,failed,ignored,measured,filtered=map(int,result[0]);assert passed>0 and failed==ignored==measured==filtered==0
            info['targets'][name]=dict(path=str(exe),sha256=sha(exe),counts=dict(passed=passed,failed=failed,ignored=ignored),test_names=re.findall(r'^test (.+) \.\.\. ok$',text,re.M));save(out/'cohort.json',state)
        assert info['targets']['native_teleport']['counts']['passed']==14
        files={'LiToModern.exe':normal,**{name+'.exe':Path(item['path'])for name,item in info['targets'].items()}}
        with zipfile.ZipFile(d/'products.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
            for name,path in files.items():z.write(path,name)
        with zipfile.ZipFile(d/'products.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path)for name,path in files.items())
        info['product_archive_sha256']=sha(d/'products.zip');guard();save(out/'cohort.json',state);print(json.dumps(dict(profile=profile,normal_sha256=info['normal']['sha256'],counts={name:item['counts']for name,item in info['targets'].items()}),ensure_ascii=False),flush=True)
    state.update(status='executed_installed_resource_profile_normal_related_both_profiles',active_profile=None)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],qualification_pending=True,actual_sessions=0),ensure_ascii=False))
