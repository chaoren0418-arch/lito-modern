"""Run candidate legacy-scope contracts against unchanged current normal public types."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211';n=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
def guard():
    q=read(ROOT/'工作佇列.json');assert not q.get('active_workloads')and not q.get('source_and_corresponding_QA_frozen')
    assert q['current_native1040_library']['source_manifest_sha256']==n['source_manifest_sha256']
    for name,h in inputs.items():assert sha(ROOT/name)==h
    for info in n['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
guard()
candidate=ROOT/'維護工具/candidates/native-legacy-audio-delivery'
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native-legacy-audio-delivery'/stamp;out.mkdir(parents=True)
runtime=P/'target/current/native-legacy-audio-delivery'/stamp;runtime.mkdir(parents=True)
pins={name:sha(candidate/name)for name in ['legacy_audio_delivery.rs','tests.rs']}
for name in pins:(out/name).write_bytes((candidate/name).read_bytes())
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
release=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['profile']['release'];assert tuple(release[k]for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    times=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in times]);ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='running_candidate_legacy_audio_scope_contracts',source_manifest_sha256=n['source_manifest_sha256'],candidate_pins=pins,
    runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],profiles={},
    normal_product_source_modified=False,normal_integration_executed=False,new_sessions=0,physical_audio_verified=False,whole_goal_complete=False)
env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),CARGO_MANIFEST_DIR=str(P))
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
def execute(args,label):
    with (out/(label+'.stdout.txt')).open('wb')as stdout,(out/(label+'.stderr.txt')).open('wb')as stderr:
        child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=args)
        state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
        job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state);assert job['exit_code']==0,label
print('LEGACY_SCOPE_RUN='+str(out),flush=True)
try:
    for profile in ['Debug','Release']:
        guard();libraries={};native=set()
        for line in (normal/profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
            if not line.startswith('{'):continue
            row=json.loads(line)
            if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
            if row.get('reason')=='compiler-artifact':
                for path in row['filenames']:
                    if path.endswith('.rlib'):
                        name=row['target']['name'].replace('-','_')
                        if name not in libraries or '/x86_64-pc-windows-msvc/'in path.replace('\\','/'):libraries[name]=path
        libraries={name:libraries[name]for name in ['lito_modern','anyhow','serde_json']}
        exe=runtime/(profile.lower()+'-legacy-scope-tests.exe')
        args=[str(rust),str(out/'tests.rs'),'--edition=2021','--test','--crate-name','native_legacy_audio_delivery_candidate','--target','x86_64-pc-windows-msvc','-o',str(exe)]
        for name,path in libraries.items():args+=['--extern',name+'='+path]
        for path in [P/('target/current/x86_64-pc-windows-msvc/'+profile.lower()+'/deps'),P/('target/current/'+profile.lower()+'/deps')]:args+=['-L','dependency='+str(path)]
        for path in sorted(native):args+=['-L',path]
        if profile=='Release':
            for key in ['opt-level','lto','codegen-units','strip']:args+=['-C',key+'='+str(release[key])]
        execute(args,profile+'-compile');guard()
        execute([str(exe),'--list'],profile+'-inventory')
        names=re.findall(r'^(.+): test$',(out/(profile+'-inventory.stdout.txt')).read_text('utf-8-sig'),re.M);assert len(names)==8
        execute([str(exe),'--test-threads=1'],profile+'-runtime')
        text=(out/(profile+'-runtime.stdout.txt')).read_text('utf-8-sig')
        assert 'test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;'in text
        assert set(re.findall(r'^test (.+) \.\.\. ok$',text,re.M))==set(names)
        state['profiles'][profile]=dict(product=str(exe),product_sha256=sha(exe),passed=8,failed=0,ignored=0,test_names=names,libraries={name:dict(path=path,sha256=sha(path))for name,path in libraries.items()})
        save(out/'receipt.json',state)
    guard();assert all(sha(candidate/name)==h for name,h in pins.items())
    state['status']='executed_both_profiles_candidate_legacy_audio_scope_contracts'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
raw={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as archive:
    for name in raw:archive.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip')as archive:assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==h for name,h in raw.items())
save(out/'readback.json',dict(files=raw,archive_sha256=sha(out/'evidence.zip')))
print(json.dumps(dict(run=str(out),profiles={p:dict(passed=8,failed=0,ignored=0)for p in state['profiles']},normal_integration_executed=False,new_sessions=0),ensure_ascii=False))
