"""Compile an isolated diagnostic against the unchanged qualified production library."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,tomllib,time,uuid
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],required=True);ap.add_argument('--g06-run',type=Path,required=True);a=ap.parse_args()
g06=a.g06_run.resolve(strict=True);assert g06.is_relative_to(ROOT/'驗證紀錄/current/native1034-G06-controller');gc=read(g06/'cohort.json')
assert gc['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases' and gc['owned_process'] is None,'no compile beside timed G06 cases'
normal=Path(gc['startup_prerequisite']);assert normal.is_relative_to(ROOT/'驗證紀錄/current/native-class2-normal');c=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035 and c['source_files']==1034
qpath=next(normal.glob(a.profile+'-qualified-*/qualified.json'));q=read(qpath)
assert q['passed']==57 and q['source_manifest_sha256']==gc['source_manifest_sha256']==c['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
assert all(j['exit_code']==0 for j in gc['process_history'])
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-'+uuid.uuid4().hex[:8];out=ROOT/'驗證紀錄/current/native1034-teleport-presentation'/('compile-'+stamp);out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-teleport-presentation'/stamp;runtime.mkdir(parents=True)
probe=ROOT/'維護工具/candidates/native-teleport-presentation/probe.rs';(out/'probe.rs').write_bytes(probe.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
libraries={};native=set()
for line in (normal/a.profile/'test-build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for p in row['filenames']:
            if p.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/' in p.replace('\\','/'):libraries[name]=p
assert {'lito_modern','anyhow','serde_json','png'}<=libraries.keys()
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';exe=runtime/'native-teleport-presentation.exe'
args=[str(rust),str(out/'probe.rs'),'--edition=2021','--crate-name','native_teleport_presentation_probe','--test','--target','x86_64-pc-windows-msvc','-L','dependency='+str(PROJECT/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps')),'-L','dependency='+str(PROJECT/('target/current/'+a.profile.lower()+'/deps')),'-o',str(exe)]
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
if a.profile=='Release':
    for key,value in [('opt-level',release['opt-level']),('lto',release['lto']),('codegen-units',release['codegen-units']),('strip',release['strip'])]:args+=['-C',key+'='+str(value)]
for n,p in sorted(libraries.items()):args+=['--extern',n+'='+p]
for p in sorted(native):args+=['-L',p]
state=dict(status='compiling_current_production_Presentation_test',profile=a.profile,source_manifest_sha256=c['source_manifest_sha256'],normal_qualification=str(qpath),normal_qualification_sha256=sha(qpath),g06_run=str(g06),runtime=str(runtime),probe_sha256=sha(probe),libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},product=str(exe),normal_product_build=False,actual_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
state['release_configuration']=release
save(out/'compiled-inputs.json',inputs);save(out/'receipt.json',state)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(PROJECT),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
print('PRESENTATION_COMPILE='+str(out),flush=True)
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    child=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    state['compiler']=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=args);save(out/'receipt.json',state)
    code=child.wait();state['compiler']['exit_code']=code
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for v in state['libraries'].values():assert sha(v['path'])==v['sha256']
state.update(status='compiled_current_production_Presentation_test' if code==0 else 'failed_preserved',product_sha256=sha(exe) if exe.exists() else None);save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],profile=a.profile,exit_code=code),ensure_ascii=False),flush=True)
if code:raise SystemExit(code)
