"""Compile a scoped controller diagnostic directly from qualified normal sources."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,tomllib,uuid
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],required=True);ap.add_argument('--normal-run',type=Path,required=True);ap.add_argument('--probe',choices=['same-map','night-audio'],default='same-map');a=ap.parse_args()
normal=a.normal_run.resolve(strict=True);assert normal.is_relative_to(ROOT/'驗證紀錄/current/native-class2-normal')
c=read(normal/'cohort.json');assert c['status']=='executed_class2_normals_related' and c['owned_process'] is None
for profile in ['Debug','Release']:
    qp=next(normal.glob(profile+'-qualified-*/qualified.json'));q=read(qp)
    assert q['status']=='independently_qualified_class2_normal_seven_related_targets' and q['passed']==57 and q['source_manifest_sha256']==c['source_manifest_sha256']
assert sha(c['profiles'][a.profile]['normal']['path'])==c['profiles'][a.profile]['normal']['sha256']
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035
for name,digest in inputs.items():assert sha(ROOT/name)==digest
app=read(Path(c['application'])/'application.json');sources={name:P/'src/modern_lito'/name for name in app['core_module_pins']}
sources['diagnostic_probe.rs']=ROOT/'維護工具/candidates/native-class2-presentation'/({'same-map':'live_probe_same_map.rs','night-audio':'live_probe_night.rs'}[a.probe])
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-'+uuid.uuid4().hex[:8]
out=ROOT/'驗證紀錄/current/native-class2-current'/('compile-'+stamp);out.mkdir(parents=True)
runtime=P/'target/current/native-class2-current'/stamp;runtime.mkdir(parents=True)
pins={name:dict(path=str(path),sha256=sha(path)) for name,path in sources.items()}
for name,path in sources.items():(out/name).write_bytes(path.read_bytes())
probe=out/'current_source.rs'
probe.write_text('#[path = '+json.dumps((P/'src/modern_lito/mod.rs').as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n#[path = '+json.dumps((P/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\n#[path = '+json.dumps((out/'diagnostic_probe.rs').as_posix(),ensure_ascii=False)+']\nmod diagnostic_probe;\nfn main()->anyhow::Result<()>{diagnostic_probe::run()}\n',encoding='utf-8')
libraries={};native=set()
for line in (normal/a.profile/'test-build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for path in row['filenames']:
            if path.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/' in path.replace('\\','/'):libraries[name]=path
assert {'lito_modern','anyhow','serde_json','png'}<=libraries.keys()
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';exe=runtime/'native-class2-current-source.exe'
args=[str(rust),str(probe),'--edition=2021','--crate-name','native_class2_current','--crate-type','bin','--target','x86_64-pc-windows-msvc','-L','dependency='+str(P/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps')),'-L','dependency='+str(P/('target/current/'+a.profile.lower()+'/deps')),'-o',str(exe)]
release=tomllib.loads((P/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
if a.profile=='Release':
    for key in ['opt-level','lto','codegen-units','strip']:args+=['-C',key+'='+str(release[key])]
for name,path in sorted(libraries.items()):args+=['--extern',name+'='+path]
for path in sorted(native):args+=['-L',path]
state=dict(status='compiling_current_class2_source_diagnostic',profile=a.profile,probe=a.probe,normal_cohort=str(normal),source_manifest_sha256=c['source_manifest_sha256'],runtime=str(runtime),product=str(exe),
    production_inputs=pins,snapshot_production_inputs={name:sha(out/name) for name in sources},libraries={name:dict(path=path,sha256=sha(path)) for name,path in libraries.items()},
    normal_product=c['profiles'][a.profile]['normal']['path'],normal_product_sha256=c['profiles'][a.profile]['normal']['sha256'],normal_GUI_product_exercised=False,
    source_overrides_used=False,release_configuration=release,normal_product_build=False,actual_sessions=0,GUI_verified=False,whole_goal_complete=False)
save(out/'compiled-inputs.json',inputs);save(out/'receipt.json',state);(out/'runner.py').write_bytes(Path(__file__).read_bytes())
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text(encoding='utf-8-sig'))['package']['version'],TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('CURRENT_SOURCE_COMPILE='+str(out),flush=True)
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    state['compiler']=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=args);save(out/'receipt.json',state)
    code=child.wait();state['compiler']['exit_code']=code
for name,digest in inputs.items():assert sha(ROOT/name)==digest
for pin in pins.values():assert sha(pin['path'])==pin['sha256']
for pin in state['libraries'].values():assert sha(pin['path'])==pin['sha256']
state.update(status='compiled_current_class2_source_diagnostic' if code==0 else 'failed_preserved',product_sha256=sha(exe) if exe.is_file() else None);save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),profile=a.profile,exit_code=code),ensure_ascii=False),flush=True)
if code:raise SystemExit(code)
