"""Metadata-only candidate typecheck bounded by observed broad compilation."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(path):
    for attempt in range(10):
        try:return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
parser=argparse.ArgumentParser();parser.add_argument('--normal-run',type=Path,required=True);parser.add_argument('--broad-run',type=Path,required=True);args=parser.parse_args()
normal=args.normal_run.resolve(strict=True);broad=args.broad_run.resolve(strict=True)
assert normal.is_relative_to(ROOT/'驗證紀錄/current/native-retired-join-normal')and broad.is_relative_to(ROOT/'驗證紀錄/current/native1040-default112')
n=read(normal/'cohort.json');c=read(broad/'cohort.json');assert n['source_manifest_sha256']==c['source_manifest_sha256']and n['source_files']==1040
qpath=next(normal.glob('Debug-qualified-*/qualified.json'));q=read(qpath);assert q['passed']==90 and q['failed']==0
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.CloseHandle.argtypes=[wintypes.HANDLE]
def birth(handle):
    values=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(value)for value in values])
    ticks=values[0].dwLowDateTime|(values[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
def compile_guard():
    state=read(broad/'cohort.json');profile=state.get('active_profile')
    assert state['status']=='executing_current_default112_profile'and profile=='Release','no typecheck beside runtime/readback'
    lines=(broad/(profile+'-console/stdout.log')).read_text('utf-8-sig').splitlines();paths=[line[len('CURRENT1040_DEFAULT112='):]for line in lines if line.startswith('CURRENT1040_DEFAULT112=')];assert len(paths)==1
    child=Path(paths[0]);r=read(child/'receipt.json');assert r['status']=='compiling_default_targets','no typecheck beside actual tests'
    job=r['owned_process'];assert '--no-run'in job['arguments']and'--release'in job['arguments']
    handle=k.OpenProcess(0x1000,False,job['pid']);assert handle
    try:observed=birth(handle)
    finally:k.CloseHandle(handle)
    assert abs((datetime.fromisoformat(observed)-datetime.fromisoformat(job['creation_utc'])).total_seconds())<.001
    return dict(profile_run=str(child),status=r['status'],owned_compiler_pid=job['pid'],creation_utc=observed)
before_guard=compile_guard()
libraries={}
for line in(normal/'Debug/test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='compiler-artifact':
        for path in row['filenames']:
            if path.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/'in path.replace('\\','/'):libraries[name]=path
selected={name:dict(path=libraries[name],sha256=sha(libraries[name]))for name in ['lito_modern','anyhow','serde_json','sha2']}
source=ROOT/'維護工具/candidates/native-eruption-assets';pins={name:sha(source/name)for name in ['probe.rs','eruption_assets.rs']}
for name,digest in inputs.items():assert sha(ROOT/name)==digest
out=ROOT/'驗證紀錄/current/native-eruption-assets'/('typecheck-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
for name in pins:(out/name).write_bytes((source/name).read_bytes())
(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',inputs)
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';metadata=out/'candidate.rmeta'
command=[str(rust),str(out/'probe.rs'),'--crate-name','native_eruption_assets_candidate','--edition=2021','--crate-type','bin','--emit=metadata','--target','x86_64-pc-windows-msvc','-L','dependency='+str(P/'target/current/x86_64-pc-windows-msvc/debug/deps'),'-L','dependency='+str(P/'target/current/debug/deps'),'-o',str(metadata)]
for name,row in selected.items():command+=['--extern',name+'='+row['path']]
state=dict(status='typechecking_candidate_metadata_only',source_manifest_sha256=c['source_manifest_sha256'],normal_qualification=str(qpath),normal_qualification_sha256=sha(qpath),normal_product_sha256=q['normal']['sha256'],candidate_inputs=pins,libraries=selected,broad_compilation_before=before_guard,command=command,runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),metadata_only=True,linked_executable=False,new_sessions=0,new_casts=0,new_renders=0,resources_loaded=False,normal_product_registered=False,normal_product_modified=False,renderer_qualified=False,whole_goal_complete=False)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('ERUPTION_ASSET_TYPECHECK='+str(out),flush=True);save(out/'receipt.json',state)
compile_guard()
with(out/'stdout.log').open('wb')as stdout,(out/'stderr.log').open('wb')as stderr:
    child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    state['compiler']=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))));save(out/'receipt.json',state)
    state['compiler']['exit_code']=child.wait()
state['broad_compilation_after']=compile_guard()
for name,digest in inputs.items():assert sha(ROOT/name)==digest
for name,digest in pins.items():assert sha(source/name)==sha(out/name)==digest
for row in selected.values():assert sha(row['path'])==row['sha256']
assert sha(q['normal']['path'])==q['normal']['sha256']
state.update(status='candidate_assets_metadata_typechecked_only'if child.returncode==0 else'failed_preserved',typechecked=child.returncode==0,metadata_sha256=sha(metadata)if metadata.exists()else None)
save(out/'receipt.json',state);files={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name in files:z.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in files.items())
save(out/'readback.json',dict(files=files,archive_sha256=sha(out/'evidence.zip')))
print(json.dumps(dict(output=str(out),exit_code=child.returncode,typechecked=state['typechecked'],metadata_only=True,resources_loaded=False,renderer_qualified=False),ensure_ascii=False),flush=True)
raise SystemExit(child.returncode)
