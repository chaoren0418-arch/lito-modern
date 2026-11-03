"""Compile a scoped controller diagnostic directly from qualified normal sources."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,tomllib,uuid,time,re
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],required=True);ap.add_argument('--normal-run',type=Path,required=True);ap.add_argument('--library-run',type=Path,required=True);ap.add_argument('--proposal',type=Path,required=True);ap.add_argument('--probe',choices=['core','live','class2'],default='core');ap.add_argument('--validation-root',type=Path);a=ap.parse_args()
normal=a.normal_run.resolve(strict=True);assert normal.is_relative_to(ROOT/'驗證紀錄/current/native-class2-normal')
c=read(normal/'cohort.json');assert c['status']=='executed_class2_normals_related' and c['owned_process'] is None
for profile in ['Debug','Release']:
    qp=next(normal.glob(profile+'-qualified-*/qualified.json'));q=read(qp)
    assert q['status']=='independently_qualified_class2_normal_seven_related_targets' and q['passed']==57 and q['source_manifest_sha256']==c['source_manifest_sha256']
assert sha(c['profiles'][a.profile]['normal']['path'])==c['profiles'][a.profile]['normal']['sha256']
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035
for name,digest in inputs.items():assert sha(ROOT/name)==digest
library=a.library_run.resolve(strict=True);assert library.is_relative_to(ROOT/'驗證紀錄/current/native1034-library-default')
def guard():
    live=read(library/'cohort.json');assert live['source_manifest_sha256']==c['source_manifest_sha256']
    if live['status']=='executed_both_current_full_library_default_selections':
        terminal=read(library/'completion-observation.json');queue=read(ROOT/'工作佇列.json')
        assert terminal['observed_terminal_exit_code']==0 and terminal['recorded_original_owned_processes_all_terminal'] and terminal['cohort_sha256']==sha(library/'cohort.json')
        assert not queue.get('active_workloads') and not queue.get('source_and_corresponding_QA_frozen')
        assert queue['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        for n,h in inputs.items():assert sha(ROOT/n)==h
        return dict(current_library_original_terminal=True,completion_sha256=sha(library/'completion-observation.json'),no_active_canonical_workloads=True)
    assert live['status']=='compiling_current_full_library_test_image','compile only alongside library compilation or after independently read terminal/checkpoint, never timed tests'
    job=live['owned_process'];pid=int(job['pid'])
    command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
    observed=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',command],creationflags=subprocess.CREATE_NO_WINDOW).decode('utf-8-sig'))
    assert observed['ProcessId']==pid and abs((datetime.fromisoformat(observed['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())<.001
    for n,h in inputs.items():assert sha(ROOT/n)==h
    return observed
observation=guard()
proposal=a.proposal.resolve(strict=True);assert proposal.is_relative_to(ROOT/'驗證紀錄/current/native-cone-cold-integration')
pj=read(proposal/'proposal.json');assert pj['status']=='prepared_unapplied_cold_scene_and_request_overlay' and pj['source_manifest_sha256']==c['source_manifest_sha256']
sources={n:proposal/'sources'/n for n in pj['proposed_sources']}
for n,h in pj['proposed_sources'].items():assert sha(sources[n])==h
probe_name={'core':'core_probe','live':'live_probe','class2':'class2_probe'}[a.probe]
validation_root=(a.validation_root or ROOT/'維護工具/candidates/native-cone-cold').resolve(strict=True)
assert validation_root.is_relative_to(ROOT/'維護工具/candidates')
sources[probe_name+'.rs']=validation_root/(probe_name+'.rs')

stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-'+uuid.uuid4().hex[:8]
out=ROOT/'驗證紀錄/current/native-cone-cold-scene'/('compile-'+stamp);out.mkdir(parents=True)
runtime=P/'target/current/native-cone-cold-scene'/stamp;runtime.mkdir(parents=True)
pins={name:dict(path=str(path),sha256=sha(path)) for name,path in sources.items()}
for name,path in sources.items():(out/name).write_bytes(path.read_bytes())
# Resolve only the module directory; no normal file is overwritten.
lines=[];previous_path=False
for line in (out/'mod.rs').read_text('utf-8-sig').splitlines():
    path=re.match(r'\s*#\[path\s*=\s*"([^"\n]+)"\]\s*$',line)
    if path:
        resolved=(P/'src/modern_lito'/path.group(1)).resolve(strict=True)
        lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']');previous_path=True;continue
    declaration=re.match(r'^(?:(?:pub(?:\([^)]*\))?)\s+)?mod\s+(\w+);\s*$',line)
    if declaration:
        name=declaration.group(1)+'.rs'
        if not previous_path:
            resolved=out/name if name in sources else P/'src/modern_lito'/name
            if not resolved.exists():resolved=P/'src/modern_lito'/declaration.group(1)/'mod.rs'
            assert resolved.is_file(),str(resolved)
            lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']')
        previous_path=False
    lines.append(line)
resolved_mod=out/'resolved_mod.rs';resolved_mod.write_text(chr(10).join(lines)+chr(10),encoding='utf-8')
probe=out/'candidate_source.rs'
wrapper='#[path = '+json.dumps(resolved_mod.as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n#[path = '+json.dumps((P/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\n'
wrapper+='pub(crate) use modern_lito::native_cone_cold;\n'
wrapper+='#[path = '+json.dumps((out/(probe_name+'.rs')).as_posix(),ensure_ascii=False)+']\nmod '+probe_name+';\n'
wrapper+=('fn main()->anyhow::Result<()>{'+probe_name+'::run()}\n' if a.probe in ['live','class2'] else 'fn main()->anyhow::Result<()>{let a:Vec<_>=std::env::args().collect();anyhow::ensure!(a.len()==3,"asset-root output");core_probe::run(std::path::Path::new(&a[1]),std::path::Path::new(&a[2]))}\n')
probe.write_text(wrapper,encoding='utf-8')
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
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';exe=runtime/'native-cone-cold-scene-source.exe'
args=[str(rust),str(probe),'--edition=2021','--crate-name','native_cone_cold_scene','--crate-type','bin','--target','x86_64-pc-windows-msvc','-L','dependency='+str(P/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps')),'-L','dependency='+str(P/('target/current/'+a.profile.lower()+'/deps')),'-o',str(exe)]
release=tomllib.loads((P/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
if a.profile=='Release':
    for key in ['opt-level','lto','codegen-units','strip']:args+=['-C',key+'='+str(release[key])]
for name,path in sorted(libraries.items()):args+=['--extern',name+'='+path]
for path in sorted(native):args+=['-L',path]
state=dict(status='compiling_isolated_cold_scene_diagnostic',profile=a.profile,probe_kind=a.probe,isolated_candidate_modules_added=True,normal_module_overrides_used=True,proposal=str(proposal),proposal_sha256=sha(proposal/'proposal.json'),library_compile_observation=observation,normal_cohort=str(normal),source_manifest_sha256=c['source_manifest_sha256'],runtime=str(runtime),product=str(exe),
    candidate_inputs=pins,snapshot_candidate_inputs={name:sha(out/name) for name in sources},libraries={name:dict(path=path,sha256=sha(path)) for name,path in libraries.items()},
    normal_product=c['profiles'][a.profile]['normal']['path'],normal_product_sha256=c['profiles'][a.profile]['normal']['sha256'],normal_GUI_product_exercised=False,
    source_overrides_used=True,release_configuration=release,normal_product_build=False,actual_sessions=0,GUI_verified=False,whole_goal_complete=False)
k0=ctypes.WinDLL('kernel32',use_last_error=True);k0.GetCurrentProcess.restype=wintypes.HANDLE;k0.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
ts0=[wintypes.FILETIME() for _ in range(4)];assert k0.GetProcessTimes(k0.GetCurrentProcess(),*[ctypes.byref(t) for t in ts0]);ticks0=ts0[0].dwLowDateTime|(ts0[0].dwHighDateTime<<32)
state['original_runner']=dict(pid=os.getpid(),creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks0//10)).isoformat())
save(out/'compiled-inputs.json',inputs);save(out/'receipt.json',state);(out/'runner.py').write_bytes(Path(__file__).read_bytes())
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text(encoding='utf-8-sig'))['package']['version'],TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('COLD_SCENE_COMPILE='+str(out),flush=True)
guard()
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    state['compiler']=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=args);save(out/'receipt.json',state)
    code=child.wait();state['compiler']['exit_code']=code
for name,digest in inputs.items():assert sha(ROOT/name)==digest
for pin in pins.values():assert sha(pin['path'])==pin['sha256']
for pin in state['libraries'].values():assert sha(pin['path'])==pin['sha256']
state.update(status='compiled_isolated_cold_scene_diagnostic' if code==0 else 'failed_preserved',product_sha256=sha(exe) if exe.is_file() else None);save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),profile=a.profile,exit_code=code),ensure_ascii=False),flush=True)
if code:raise SystemExit(code)
