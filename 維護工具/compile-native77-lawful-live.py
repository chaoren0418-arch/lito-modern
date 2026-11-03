"""Build an owned diagnostic wrapper linked to the current production Debug library."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1]; PROJECT=ROOT/'LiTo381-Modern'
RUN=ROOT/'驗證紀錄/current/native77-lawful/validation-20261017-195458-430865'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
c=read(RUN/'cohort.json'); inputs=read(RUN/'compiled-inputs.json')
assert c['owned_process'] is None or c['owned_process']['label'].endswith(('normal-build','test-build')), 'no compile beside timed runtime cases'
q=read(next(RUN.glob('Debug-qualified-*/qualified.json')))
assert q['passed']==31 and q['source_manifest_sha256']==c['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native77-live-compile'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native77-live'/stamp;runtime.mkdir(parents=True)
probe=ROOT/'維護工具/candidates/native77-live/probe.rs'
(out/'probe.rs').write_bytes(probe.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
wrapper=out/'wrapper.rs';wrapper.write_text('mod modern_lito {pub use lito_modern::modern_lito::*;}\n#[path="'+(out/'probe.rs').as_posix()+'"]\nmod probe;\nfn main()->anyhow::Result<()>{probe::run()}\n',encoding='utf-8')
libraries={};native=set()
for line in (RUN/'Debug/test-build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for p in row['filenames']:
            if p.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/' in p.replace('\\','/'):libraries[name]=p
assert {'lito_modern','anyhow','serde_json','png'}<=libraries.keys()
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
exe=runtime/'native77-live.exe'
args=[str(rust),str(wrapper),'--edition=2021','--crate-name','native77_live_probe','--crate-type','bin','--target','x86_64-pc-windows-msvc','-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
for n,p in sorted(libraries.items()):args+=['--extern',n+'='+p]
for p in sorted(native):args+=['-L',p]
state=dict(status='compiling_current_production_library_wrapper',source_manifest_sha256=c['source_manifest_sha256'],normal_qualification=str(next(RUN.glob('Debug-qualified-*/qualified.json'))),runtime=str(runtime),probe_sha256=sha(probe),libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},product=str(exe),normal_product_build=False,actual_sessions=0,actual_casts=0,GUI_verified=False,whole_goal_complete=False)
save(out/'compiled-inputs.json',inputs);save(out/'receipt.json',state)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(PROJECT),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
print('NATIVE77_LIVE_COMPILE='+str(out),flush=True)
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    child=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    state['compiler']=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=args);save(out/'receipt.json',state)
    code=child.wait();state['compiler']['exit_code']=code
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for v in state['libraries'].values():assert sha(v['path'])==v['sha256']
state.update(status='compiled_current_production_library_wrapper' if code==0 else 'failed_preserved',product_sha256=sha(exe) if exe.exists() else None)
save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],exit_code=code),ensure_ascii=False),flush=True)
if code:raise SystemExit(code)
