"""Build two nonempty-audio probes from current normal bodies and one pinned API suffix."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,time,tomllib,uuid
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current';T=ROOT/'維護工具'
def read(p):
    for attempt in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],required=True);ap.add_argument('--g06-run',type=Path,required=True);a=ap.parse_args()
g06=a.g06_run.resolve(strict=True);assert g06.is_relative_to(V/'native1040-G06-controller')
gc=read(g06/'cohort.json');go=read(g06/'completion-observation.json');gq=g06/'qualified/qualified.json'
assert gc['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases' and gc['owned_process'] is None
assert go['observed_terminal_exit_code']==0 and go['recorded_original_owned_processes_all_terminal'] and go['qualification_sha256']==sha(gq)
assert read(gq)['actual_formal_sessions']==96
normal=Path(gc['startup_prerequisite']);assert normal.is_relative_to(V/'native-legacy-audio-normal')
c=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041 and c['source_files']==1040
assert c['source_manifest_sha256']==gc['source_manifest_sha256']=='8CC5A560EFC60C6830658A06B3C4430593493377D433684090B4F46F3427D24E'
qp=next(normal.glob(a.profile+'-qualified-*/qualified.json'));q=read(qp);assert(q['passed'],q['failed'],q['ignored'])==(130,0,0) and len(q['targets'])==17
candidate=T/'candidates/native8CC5-teleport-audio';prepared=read(candidate/'preparation.json');assert prepared['source_manifest_sha256']==c['source_manifest_sha256']
for n,h in prepared['files'].items():assert sha(candidate/n)==h
binding=V/'native-rental-action-wire-binding/20261018-093155-010472/binding.json'
bp=read(binding);assert bp['request_opcode']==31 and bp['request_action']==68 and bp['selected_sound']==1534
reference=P/'target/current/server-reference/formal-server-C02AAAE8.jar'
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
formal_sha='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
def guard():
    queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and queue.get('source_and_corresponding_QA_frozen')
    assert queue['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for info in c['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
    for n,h in prepared['files'].items():assert sha(candidate/n)==h
    assert sha(formal)==sha(reference)==formal_sha
guard()
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS'})}
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-'+uuid.uuid4().hex[:8]
out=V/'native8CC5-teleport-audio'/('compile-'+stamp);out.mkdir(parents=True)
runtime=P/'target/current/native8CC5-teleport-audio'/stamp;runtime.mkdir(parents=True)
source_paths={'mod.rs':P/'src/modern_lito/mod.rs','native_login.original.rs':P/'src/modern_lito/native_login.rs',
    **{n:candidate/n for n in prepared['files']}}
pins={n:dict(path=str(path),sha256=sha(path)) for n,path in source_paths.items()}
for n,path in source_paths.items():(out/n).write_bytes(path.read_bytes())
original=(out/'native_login.original.rs').read_bytes();suffix=b'\n'+(out/'diagnostic_wave_api.rs').read_bytes()
(out/'native_login.rs').write_bytes(original+suffix)
lines=[];previous_path=False;overrides=0
for line in (out/'mod.rs').read_text('utf-8-sig').splitlines():
    path=re.match(r'\s*#\[path\s*=\s*"([^"\n]+)"\]\s*$',line)
    if path:
        resolved=(P/'src/modern_lito'/path.group(1)).resolve(strict=True)
        lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']');previous_path=True;continue
    declaration=re.match(r'^(?:(?:pub(?:\([^)]*\))?)\s+)?mod\s+(\w+);\s*$',line)
    if declaration:
        name=declaration.group(1)+'.rs'
        if not previous_path:
            resolved=out/name if name=='native_login.rs' else P/'src/modern_lito'/name
            if name=='native_login.rs':overrides+=1
            if not resolved.exists():resolved=P/'src/modern_lito'/declaration.group(1)/'mod.rs'
            assert resolved.is_file(),str(resolved)
            lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']')
        previous_path=False
    lines.append(line)
assert overrides==1
(out/'resolved_mod.rs').write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')
wrapper='#[path = '+json.dumps((out/'resolved_mod.rs').as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n'
wrapper+='#[path = '+json.dumps((P/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\n'
(out/'lib_source.rs').write_text(wrapper,encoding='utf-8',newline='\n')
libraries={};native=set();development=[]
for line in (normal/a.profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        if row['target']['name']=='lito_modern' and not row['profile']['test']:development.append(row['profile'])
        for path in row['filenames']:
            if path.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/' in path.replace('\\','/'):libraries[name]=path
assert {'lito_modern','anyhow','serde_json','png','zeroize'}<=libraries.keys();libraries.pop('lito_modern')
release=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['profile']['release'];assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
common=['--edition=2021','--target','x86_64-pc-windows-msvc']
for path in [P/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps'),P/('target/current/'+a.profile.lower()+'/deps')]:common+=['-L','dependency='+str(path)]
for name,path in sorted(libraries.items()):common+=['--extern',name+'='+path]
for path in sorted(native):common+=['-L',path]
if a.profile=='Release':
    for key in ['opt-level','lto','codegen-units','strip']:common+=['-C',key+'='+str(release[key])]
else:
    assert development and all(p['opt_level']=='0' and p['debuginfo']==1 and p['debug_assertions'] and p['overflow_checks'] for p in development)
    common+=['-C','opt-level=0','-C','debuginfo=1','-C','debug-assertions=yes','-C','overflow-checks=yes']
rlib=runtime/'liblito_modern_wave_diagnostic.rlib'
products={kind:dict(path=str(runtime/(kind+'-audio.exe'))) for kind in ['success','failure']}
commands=[('library',[str(rust),str(out/'lib_source.rs'),'--crate-name','lito_modern','--crate-type','rlib','-C','metadata=normal8cc5_wave_diagnostic','-o',str(rlib)]+common)]
commands += [(kind,[str(rust),str(out/(kind+'.rs')),'--crate-name','native_teleport_'+kind+'_audio','--test','--extern','lito_modern='+str(rlib),'-o',products[kind]['path']]+common) for kind in products]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
identity=dict(normal_source_manifest_sha256=c['source_manifest_sha256'],diagnostic_API_sha256=prepared['files']['diagnostic_wave_api.rs'],probe_sha256={kind:prepared['files'][kind+'.rs'] for kind in products},source_overlays_used=False)
save(out/'diagnostic-inputs.json',identity)
state=dict(status='compiling_current_normal_body_audio_diagnostics',profile=a.profile,source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,
    normal_cohort=str(normal),normal_qualification=str(qp),normal_qualification_sha256=sha(qp),g06_run=str(g06),g06_qualification_sha256=sha(gq),
    original_runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),candidate_inputs=pins,
    source_overlays_used=False,normal_source_bodies_unchanged=True,normal_module_snapshot_extension_used=True,diagnostic_request_API_extension=True,
    native_login_original_bytes=len(original),native_login_original_sha256=sha(out/'native_login.original.rs'),
    appended_API_bytes=len(suffix),appended_API_sha256=hashlib.sha256(suffix).hexdigest().upper(),diagnostic_native_login_sha256=sha(out/'native_login.rs'),
    diagnostic_input_manifest_sha256=sha(out/'diagnostic-inputs.json'),formal_C1F_binding=dict(path=str(binding),sha256=sha(binding)),
    normal_GUI_API_permission_inferred=False,normal_product_source_modified=False,normal_EXE_runtime_inferred=False,
    libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},diagnostic_library=str(rlib),products=products,
    release_configuration=release,process_history=[],owned_process=None,actual_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'compiled-inputs.json',inputs);(out/'source-inputs.json').write_bytes((normal/'source-inputs.json').read_bytes());assert sha(out/'source-inputs.json')==c['source_manifest_sha256']
(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'receipt.json',state)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('NORMAL_AUDIO_COMPILE='+str(out),flush=True)
try:
    for label,args in commands:
        guard()
        with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:
            child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=args)
            state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state);assert job['exit_code']==0,'Preserve '+label+' compiler failure'
    guard();assert(out/'native_login.rs').read_bytes()==original+suffix
    for pin in pins.values():assert sha(pin['path'])==pin['sha256']
    for pin in state['libraries'].values():assert sha(pin['path'])==pin['sha256']
    for product in products.values():product['sha256']=sha(product['path'])
    state.update(status='compiled_current_normal_body_audio_diagnostics',diagnostic_library_sha256=sha(rlib))
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(compile=str(out),profile=a.profile,products={k:v['sha256'] for k,v in products.items()},normal_product_source_modified=False),ensure_ascii=False))
