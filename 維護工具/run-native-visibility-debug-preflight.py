"""Compile the complete product modules, run Debug gates, then unique entry."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def retain(p,files):
    hashes={n:sha(f) for n,f in files.items()}
    with zipfile.ZipFile(p,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,f in files.items():z.write(f,n)
    with zipfile.ZipFile(p) as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for n,h in hashes.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    return dict(path=str(p),sha256=sha(p),files=hashes)
ap=argparse.ArgumentParser();ap.add_argument('application',type=Path)
ap.add_argument('--input-snapshot',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],default='Debug');a=ap.parse_args();profile=a.profile
app=a.application.resolve(strict=True);assert app.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
applied=read(app/'apply-receipt.json');inputs=read(app/'applied-compiled-inputs.json')
assert len(inputs)==1025 and applied['source_manifest_sha256']==sha(app/'applied-source-inputs.json')
source_path=app/'applied-source-inputs.json';compiled_path=app/'applied-compiled-inputs.json'
repair=None
if a.input_snapshot:
    snapshot=a.input_snapshot.resolve(strict=True);assert snapshot.is_relative_to(app)
    if (snapshot/'integration.json').exists():
        integration=read(snapshot/'integration.json');assert Path(integration['application']).resolve()==app
        parent=Path(integration['parent_snapshot']).resolve(strict=True);assert parent.is_relative_to(app)
        parent_inputs=read(parent/'compiled-inputs.json');fixed=read(snapshot/'compiled-inputs.json')
        expected=['LiTo381-Modern/src/modern_lito/native_peer_probe.rs','LiTo381-Modern/src/modern_lito/native_peer_actions.rs']
        assert set(fixed)==set(parent_inputs) and set(n for n in fixed if fixed[n]!=parent_inputs[n])==set(expected)
        assert integration['changed_paths']==expected
        with zipfile.ZipFile(snapshot/'before.zip') as z:
            assert set(z.namelist())==set(expected)
            for n in expected:assert hashlib.sha256(z.read(n)).hexdigest().upper()==parent_inputs[n]
        assert sha(snapshot/'before.zip')==integration['before_archive_sha256']
        assert sha(snapshot/'source-inputs.json')==integration['source_manifest_sha256']
        source_path=snapshot/'source-inputs.json';compiled_path=snapshot/'compiled-inputs.json';inputs=fixed
        snapshot_for_reference=parent
    else:snapshot_for_reference=snapshot
    repair=read(snapshot_for_reference/'repair.json');assert Path(repair['application']).resolve()==app
    assert repair['old_source_manifest_sha256']==applied['source_manifest_sha256']
    failed=Path(repair['previous_failed_run']).resolve(strict=True)
    assert failed.is_relative_to(app.parent) and read(failed/'cohort.json')['status']=='failed_preserved'
    assert sha(failed/'failure-raw.zip')==repair['previous_failure_archive_sha256']
    reference_inputs=read(app/'applied-compiled-inputs.json');fixed=read(snapshot_for_reference/'compiled-inputs.json')
    assert set(fixed)==set(reference_inputs) and [n for n in reference_inputs if reference_inputs[n]!=fixed[n]]==['LiTo381-Modern/tests/native_skills.rs']
    assert repair['runtime_source_files_changed'] is False and sha(snapshot_for_reference/'source-inputs.json')==repair['source_manifest_sha256']
    assert sha(snapshot_for_reference/'before-native_skills.rs')==repair['before_test_sha256']==reference_inputs['LiTo381-Modern/tests/native_skills.rs']
    assert fixed['LiTo381-Modern/tests/native_skills.rs']==repair['after_test_sha256']
    if snapshot_for_reference==snapshot:
        inputs=fixed;source_path=snapshot/'source-inputs.json';compiled_path=snapshot/'compiled-inputs.json'
source_manifest_sha=sha(source_path)
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
out=app.parent/(profile+'-preflight-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
(out/'source-inputs.json').write_bytes(source_path.read_bytes())
(out/'compiled-inputs.json').write_bytes(compiled_path.read_bytes())
if repair:(out/'reference-repair.json').write_bytes((snapshot_for_reference/'repair.json').read_bytes())
if a.input_snapshot and (snapshot/'integration.json').exists():(out/'probe-integration.json').write_bytes((snapshot/'integration.json').read_bytes())
targets=['native_skills','native_skill_selection','native_actor_states']
entry_sha=sha(ROOT/'啟動現代客戶端.ps1')
state=dict(status=f'prepared_product_visibility_{profile}_preflight',output=str(out),application=str(app),
    profile=profile,source_files=1024,compiled_input_files=1025,source_manifest_sha256=source_manifest_sha,
    input_snapshot=str(snapshot) if repair else None,
    entry_sha256=entry_sha,targets=targets,tests={},products={},process_history=[],owned_process=None,
    source_and_runner_frozen=True,actual_formal_sessions=0,actual_casts=0,GUI_verified=False,whole_goal_complete=False)
def persist():save(out/'cohort.json',state)
def check():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    assert sha(ROOT/'啟動現代客戶端.ps1')==entry_sha
env={k:v for k,v in os.environ.items() if not k.startswith('LITO_')}
env.update(PATH=str(RUST)+os.pathsep+env['PATH'],RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),
    TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1')
def execute(command,label):
    d=out/label;d.mkdir();check()
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];k=ctypes.WinDLL('kernel32',use_last_error=True)
        k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
        assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
        tick=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        item=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=tick//10)).isoformat(),arguments=command,output=str(d))
        state['process_history'].append(item);state['owned_process']=item;persist()
        code=p.wait();item.update(exit_code=code,finished_utc=datetime.now(timezone.utc).isoformat());state['owned_process']=None;persist()
    check();return code
check();state['compiled_source_archive']=retain(out/'compiled-source.zip',{n:ROOT/n for n in inputs});persist()
print(json.dumps(dict(output=str(out),source_manifest_sha256=state['source_manifest_sha256']),ensure_ascii=False),flush=True)
try:
    state['status']=f'compiling_full_product_visibility_{profile}_targets';persist()
    command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--bin','LiToModern']
    if profile=='Release':command.append('--release')
    for target in targets:command+=['--test',target]
    assert execute(command,'build')==0
    messages=[json.loads(l) for l in (out/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
    artifacts=[m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']]
    assert len(artifacts)==4 and {m['target']['name'] for m in artifacts}==set(targets)|{'LiToModern'}
    for m in artifacts:
        assert m['features']==[];p=Path(m['executable']).resolve(strict=True);assert p.is_relative_to(PROJECT/'target/current')
        state['products'][m['target']['name']]=dict(path=str(p),sha256=sha(p),compiler_profile=m['profile'])
    state['test_products_archive']=retain(out/'test-products.zip',{n+'.exe':Path(v['path']) for n,v in state['products'].items()});persist()
    state['status']=f'executing_full_product_visibility_{profile}_targets';persist()
    for n,product in state['products'].items():
        code=execute([product['path'],'--test-threads=1'],'test-'+n)
        text=(out/('test-'+n)/'stdout.log').read_text(encoding='utf-8-sig')
        rows=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
        assert len(rows)==1
        status,passed,failed,ignored,measured,filtered=rows[0]
        state['tests'][n]=dict(exit_code=code,status=status,passed=int(passed),failed=int(failed),ignored=int(ignored),measured=int(measured),filtered=int(filtered));persist()
        assert code==0 and status=='ok' and int(failed)==int(measured)==int(filtered)==0,'Preserve actual failure before repair'
    state['counts']={k:sum(v[k] for v in state['tests'].values()) for k in ['passed','failed','ignored']};persist()
    state['status']=f'building_unique_entry_visibility_{profile}_normal';persist()
    assert execute(['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile],'normal-build')==0
    (out/'entry-build.log').write_bytes((ROOT/'驗證紀錄/current/build.log').read_bytes())
    exe=PROJECT/('target/current/x86_64-pc-windows-msvc/'+profile.lower()+'/LiToModern.exe')
    state['normal']=dict(path=str(exe),sha256=sha(exe),unique_entry_exit_code=0,archive=retain(out/'normal-product.zip',{'LiToModern.exe':exe}));persist()
    assert execute([str(exe),'--version'],'version')==0
    state.update(status=f'executed_product_visibility_{profile}_preflight_and_unique_normal',source_and_runner_frozen=False)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    state['finished_utc']=datetime.now(timezone.utc).isoformat();persist()
print(json.dumps(dict(output=str(out),status=state['status'],counts=state.get('counts'),normal=state.get('normal',{}).get('sha256')),ensure_ascii=False),flush=True)
