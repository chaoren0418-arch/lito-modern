"""Compile an isolated diagnostic against the unchanged qualified production library."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,tomllib,uuid
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],required=True);ap.add_argument('--g06-run',type=Path,required=True);ap.add_argument('--diagnostic-entry',type=Path);a=ap.parse_args()
broad=ROOT/'驗證紀錄/current/default-native-regression/20261017-221209-558-Release/receipt.json'
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(ROOT)
g06=a.g06_run.resolve(strict=True);assert g06.is_relative_to(ROOT/'驗證紀錄/current/native-G06-controller');gc=read(g06/'cohort.json')
assert gc['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases' and gc['owned_process'] is None,'no compile beside timed G06 cases'
normal=Path(gc['startup_prerequisite']);c=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json')
qpath=next(normal.glob(a.profile+'-qualified-*/qualified.json'));q=read(qpath)
assert q['passed']==31 and q['source_manifest_sha256']==gc['source_manifest_sha256']==c['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
assert all(j['exit_code']==0 for j in gc['process_history'])
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-'+uuid.uuid4().hex[:8];out=ROOT/'驗證紀錄/current/native-class2-presentation'/('compile-'+stamp);out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-class2-presentation'/stamp;runtime.mkdir(parents=True)
sources={
    'native_combat_animation.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_combat_animation.rs',
    'native_world_animation.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_world_animation.rs',
    'native_environment.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_environment.rs',
    'native_skills.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_skills.rs',
    'native_skill_selection.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_skill_selection.rs',
    'native_world_scene.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_world_scene.rs',
    'native_world_view.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_world_view.rs',
    'native_login.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_login.rs',
    'native_login_window.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_login_window.rs',
    'native_ambient_bridge.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_ambient_bridge.rs',
    'call_lightning_assets.rs':ROOT/'維護工具/candidates/native-class2-presentation/call_lightning_assets.rs',
    'native_call_lightning.rs':ROOT/'維護工具/candidates/native-class2-presentation/native_call_lightning.rs',
    'native_ambient_override.rs':ROOT/'維護工具/candidates/native-class2-ambient/native_ambient_override.rs'}
if a.diagnostic_entry:
    entry=a.diagnostic_entry.resolve(strict=True);assert entry.is_relative_to(ROOT/'維護工具/candidates/native-class2-presentation');sources['diagnostic_probe.rs']=entry
candidate_pins={name:dict(path=str(path),sha256=sha(path)) for name,path in sources.items()}
for name,path in sources.items():
    snapshot=path.read_text(encoding='utf-8-sig')
    if (PROJECT/'src/modern_lito'/name).is_file():
        import re
        snapshot=re.sub(r'#\[path\s*=\s*"([^"]+)"\]',lambda m:'#[path = '+json.dumps((PROJECT/'src/modern_lito'/m[1]).as_posix(),ensure_ascii=False)+']',snapshot)
    (out/name).write_text(snapshot,encoding='utf-8',newline='\n')
snapshot_pins={name:sha(out/name) for name in sources}
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
module=PROJECT/'src/modern_lito/mod.rs'
text=module.read_text(encoding='utf-8-sig')
import re
text=re.sub(r'#\[path\s*=\s*"([^"]+)"\]',lambda m:'#[path = '+json.dumps((module.parent/m[1]).as_posix(),ensure_ascii=False)+']',text)
lines=[];has_path=False
for line in text.splitlines():
    if '#[path' in line:has_path=True
    match=re.fullmatch(r'\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+);\s*',line)
    if match:
        if not has_path:
            target=out/(match[1]+'.rs') if match[1]+'.rs' in sources else module.parent/(match[1]+'.rs')
            if not target.is_file():target=module.parent/match[1]/'mod.rs'
            assert target.is_file(),line
            lines.append('#[path = '+json.dumps(target.as_posix(),ensure_ascii=False)+']')
        has_path=False
    lines.append(line)
declared=set(re.findall(r'(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+);',text))
for name in sources:
    if Path(name).stem in declared or name=='diagnostic_probe.rs':continue
    lines+=['#[path = '+json.dumps((out/name).as_posix(),ensure_ascii=False)+']','pub mod '+Path(name).stem+';']
(out/'modern_lito_overlay.rs').write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')
probe=out/'candidate_lib.rs'
probe.write_text('#[path = '+json.dumps((out/'modern_lito_overlay.rs').as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n#[path = '+json.dumps((PROJECT/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\n',encoding='utf-8',newline='\n')
if a.diagnostic_entry:
    with probe.open('a',encoding='utf-8',newline='\n') as stream:stream.write('#[path = '+json.dumps((out/'diagnostic_probe.rs').as_posix(),ensure_ascii=False)+']\nmod diagnostic_probe;\nfn main()->anyhow::Result<()>{diagnostic_probe::run()}\n')
candidate_hash=sha(out/'call_lightning_assets.rs')
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
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';exe=runtime/('native-class2-diagnostic.exe' if a.diagnostic_entry else 'native-class2-typecheck.rmeta')
args=[str(rust),str(probe),'--edition=2021','--crate-name','native_class2_typecheck','--crate-type','lib','--emit','metadata','--target','x86_64-pc-windows-msvc','-L','dependency='+str(PROJECT/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps')),'-L','dependency='+str(PROJECT/('target/current/'+a.profile.lower()+'/deps')),'-o',str(exe)]
if a.diagnostic_entry:
    index=args.index('--crate-type');args[index+1]='bin';index=args.index('--emit');del args[index:index+2]
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
if a.profile=='Release':
    for key,value in [('opt-level',release['opt-level']),('lto',release['lto']),('codegen-units',release['codegen-units']),('strip',release['strip'])]:args+=['-C',key+'='+str(value)]
for n,p in sorted(libraries.items()):args+=['--extern',n+'='+p]
for p in sorted(native):args+=['-L',p]
state=dict(status='compiling_current_class2_candidate_typecheck',profile=a.profile,source_manifest_sha256=c['source_manifest_sha256'],normal_qualification=str(qpath),normal_qualification_sha256=sha(qpath),g06_run=str(g06),runtime=str(runtime),probe_sha256=sha(probe),libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},product=str(exe),normal_product_build=False,actual_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
state['release_configuration']=release
state['diagnostic_executable']=bool(a.diagnostic_entry)
state.update(candidate_module_sha256=candidate_hash,compiled_probe_sha256=sha(probe),candidate_inputs=candidate_pins,snapshot_candidate_inputs=snapshot_pins,generated_overlay_sha256=sha(out/'modern_lito_overlay.rs'),candidate_only=True,real_casts_verified=False,renderer_verified=False)
save(out/'compiled-inputs.json',inputs);save(out/'receipt.json',state)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(PROJECT),CARGO_PKG_VERSION=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['package']['version'],TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
print('CLASS2_TYPECHECK='+str(out),flush=True)
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    child=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    state['compiler']=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=args);save(out/'receipt.json',state)
    code=child.wait();state['compiler']['exit_code']=code
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for name,pin in candidate_pins.items():
    assert sha(pin['path'])==pin['sha256'] and sha(out/name)==snapshot_pins[name]
for v in state['libraries'].values():assert sha(v['path'])==v['sha256']
state.update(status=('compiled_current_class2_candidate_diagnostic' if a.diagnostic_entry else 'compiled_current_class2_candidate_typecheck') if code==0 else 'failed_preserved',product_sha256=sha(exe) if exe.exists() else None);save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],profile=a.profile,exit_code=code),ensure_ascii=False),flush=True)
if code:raise SystemExit(code)
