"""Build a source-body diagnostic rlib with a separately pinned C1F request extension."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, re, subprocess, time, tomllib, uuid

ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):
    for attempt in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
parser=argparse.ArgumentParser();parser.add_argument('--profile',required=True,choices=['Debug','Release']);parser.add_argument('--overlay-dir',type=Path,required=True);parser.add_argument('--probe-dir',type=Path,required=True);a=parser.parse_args()
normal=V/'native-retired-join-normal/validation-20261018-063154-317211';c=read(normal/'cohort.json')
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041 and c['source_files']==1040
qpath=next(normal.glob(a.profile+'-qualified-*/qualified.json'));q=read(qpath);assert q['passed']==90 and q['failed']==0
overlay=a.overlay_dir.resolve(strict=True);assert overlay.is_relative_to(ROOT/'維護工具/candidates/native-legacy-audio-lifecycle')
overlay_record=read(overlay/'preparation.json');assert overlay_record['normal_source_manifest_sha256']==c['source_manifest_sha256']
overlays=overlay_record['source_overlays'];assert len(overlays)==4
for item in overlays.values():assert sha(item['original_path'])==item['original_sha256'] and sha(item['candidate_path'])==item['candidate_sha256']
candidate=a.probe_dir.resolve(strict=True);assert candidate.is_relative_to(ROOT/'維護工具/candidates');prepared=read(candidate/'preparation.json')
assert prepared['normal_baseline_source_manifest_sha256']==c['source_manifest_sha256']==q['source_manifest_sha256']
assert sha(candidate/'probe.rs')==prepared['candidate_probe_sha256'] and sha(candidate/'diagnostic_wave_api.rs')==prepared['diagnostic_API_sha256']
binding_path=V/'native-rental-action-wire-binding/20261018-093155-010472/binding.json';binding=dict(path=str(binding_path),sha256=sha(binding_path))
primary=read(binding['path']);assert primary['request_opcode']==31 and primary['request_action']==68 and primary['selected_sound']==1534
def guard():
    queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and not queue.get('source_and_corresponding_QA_frozen')
    library=queue['current_native1040_library'];assert library['source_manifest_sha256']==c['source_manifest_sha256']
    assert library['observed_actual_exit_code']==0 and library['recorded_original_owned_processes_all_terminal']
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for profile,info in c['profiles'].items():assert sha(info['normal']['path'])==info['normal']['sha256']
guard()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-'+uuid.uuid4().hex[:8]
out=V/'native1040-teleport-legacy-audio'/('compile-'+stamp);out.mkdir(parents=True)
runtime=P/'target/current/native1040-teleport-legacy-audio'/stamp;runtime.mkdir(parents=True)
sources={'mod.rs':P/'src/modern_lito/mod.rs','native_login.original.rs':P/'src/modern_lito/native_login.rs',
    'probe.rs':candidate/'probe.rs','diagnostic_wave_api.rs':candidate/'diagnostic_wave_api.rs'}
sources.update({name:Path(item['candidate_path'])for name,item in overlays.items()})
pins={name:dict(path=str(path),sha256=sha(path))for name,path in sources.items()}
for name,path in sources.items():(out/name).write_bytes(path.read_bytes())
relocations={}
for name,item in overlays.items():
    parent=Path(item['original_path']).parent;changed=[]
    def relocate(match):
        target=(parent/match.group(1)).resolve(strict=True)
        assert target.is_file()
        changed.append(dict(relative_path=match.group(1),path=str(target),sha256=sha(target)))
        return '#[path = '+json.dumps(target.as_posix(),ensure_ascii=False)+']'
    snapshot=re.sub(r'#\[path\s*=\s*"([^"\n]+)"\]',relocate,(out/name).read_text('utf-8-sig'))
    (out/name).write_text(snapshot,encoding='utf-8');relocations[name]=changed
original=(out/'native_login.original.rs').read_bytes();suffix=b'\n'+(out/'diagnostic_wave_api.rs').read_bytes()
(out/'native_login.rs').write_bytes(original+suffix)
assert (out/'native_login.rs').read_bytes()==original+suffix
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
            resolved=out/name if name=='native_login.rs' or name in overlays else P/'src/modern_lito'/name
            if name=='native_login.rs' or name in overlays:overrides+=1
            if not resolved.exists():resolved=P/'src/modern_lito'/declaration.group(1)/'mod.rs'
            assert resolved.is_file(),str(resolved)
            lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']')
        previous_path=False
    lines.append(line)
assert overrides==5
(out/'resolved_mod.rs').write_text('\n'.join(lines)+'\n',encoding='utf-8')
wrapper='#[path = '+json.dumps((out/'resolved_mod.rs').as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n'
wrapper+='#[path = '+json.dumps((P/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\n'
(out/'lib_source.rs').write_text(wrapper,encoding='utf-8')
libraries={};native=set()
for line in (normal/a.profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for path in row['filenames']:
            if path.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/'in path.replace('\\','/'):libraries[name]=path
assert {'lito_modern','anyhow','serde_json','png','zeroize'}<=libraries.keys()
libraries.pop('lito_modern')
release=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['profile']['release']
assert tuple(release[k]for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
common=['--edition=2021','--target','x86_64-pc-windows-msvc']
for path in [P/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps'),P/('target/current/'+a.profile.lower()+'/deps')]:common+=['-L','dependency='+str(path)]
for name,path in sorted(libraries.items()):common+=['--extern',name+'='+path]
for path in sorted(native):common+=['-L',path]
if a.profile=='Release':
    for key in ['opt-level','lto','codegen-units','strip']:common+=['-C',key+'='+str(release[key])]
else:
    # Match the observed current Cargo development artifact, including its
    # configured line-table debug information; retain all debug checks.
    profiles=[]
    for line in (normal/'Debug/test-build/stdout.log').read_text('utf-8-sig').splitlines():
        if not line.startswith('{'):continue
        row=json.loads(line)
        if row.get('reason')=='compiler-artifact'and row['target']['name']=='lito_modern'and not row['profile']['test']:profiles.append(row['profile'])
    assert profiles and all(p['opt_level']=='0'and p['debuginfo']==1 and p['debug_assertions']and p['overflow_checks']for p in profiles)
    common+=['-C','opt-level=0','-C','debuginfo=1','-C','debug-assertions=yes','-C','overflow-checks=yes']
rlib=runtime/'liblito_modern_legacy_lifecycle.rlib';exe=runtime/'native-teleport-action-audio.exe'
commands=[('library',[str(rust),str(out/'lib_source.rs'),'--crate-name','lito_modern','--crate-type','rlib','-C','metadata=legacy_lifecycle_candidate','-o',str(rlib)]+common),
    ('probe',[str(rust),str(out/'probe.rs'),'--crate-name','native_teleport_legacy_audio','--test','--extern','lito_modern='+str(rlib),'-o',str(exe)]+common)]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    times=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in times]);ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(snapshot_path_relocations=relocations,snapshot_overlay_sha256={name:sha(out/name)for name in overlays},metadata_only=False,source_overlay_manifest=dict(path=str(overlay/'preparation.json'),sha256=sha(overlay/'preparation.json')),source_overlays=overlays,status='compiling_candidate_legacy_audio_lifecycle_library_and_probe',profile=a.profile,source_manifest_sha256=c['source_manifest_sha256'],
    source_files=1040,compiled_input_files=1041,normal_qualification=str(qpath),normal_qualification_sha256=sha(qpath),
    normal_cohort=str(normal),g06_run=str(V/'native1040-G06-controller/20261018-070120-391344'),
    original_runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),candidate_inputs=pins,
    native_login_original_bytes=len(original),native_login_original_sha256=hashlib.sha256(original).hexdigest().upper(),
    appended_API_bytes=len(suffix),appended_API_sha256=hashlib.sha256(suffix).hexdigest().upper(),
    diagnostic_native_login_sha256=sha(out/'native_login.rs'),diagnostic_request_API_extension=True,
    normal_source_bodies_unchanged=False,normal_module_snapshot_extension_used=True,normal_GUI_API_permission_inferred=False,
    libraries={name:dict(path=path,sha256=sha(path))for name,path in libraries.items()},diagnostic_library=str(rlib),
    product=str(exe),normal_product_source_modified=False,normal_EXE_runtime_inferred=False,release_configuration=release,
    probe_sha256=sha(candidate/'probe.rs'),process_history=[],actual_sessions=0,GUI_verified=False,whole_goal_complete=False)
identity=dict(normal_baseline_source_manifest_sha256=c['source_manifest_sha256'],source_overlays={name:item['candidate_sha256']for name,item in overlays.items()},diagnostic_API_sha256=prepared['diagnostic_API_sha256'],probe_sha256=prepared['candidate_probe_sha256'])
identity_bytes=(json.dumps(identity,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode('utf-8');(out/'candidate-effective-inputs.json').write_bytes(identity_bytes)
state['candidate_effective_manifest_sha256']=hashlib.sha256(identity_bytes).hexdigest().upper();state['candidate_effective_inputs']=identity
save(out/'compiled-inputs.json',inputs);save(out/'receipt.json',state);(out/'runner.py').write_bytes(Path(__file__).read_bytes())
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('LEGACY_LIFECYCLE_COMPILE='+str(out),flush=True)
try:
    for label,args in commands:
        guard()
        with (out/(label+'.stdout.log')).open('wb')as stdout,(out/(label+'.stderr.log')).open('wb')as stderr:
            child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=args)
            state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            code=child.wait();job['exit_code']=code;state['owned_process']=None;save(out/'receipt.json',state)
            assert code==0,'Preserve original '+label+' compilation failure'
    guard()
    assert (out/'native_login.rs').read_bytes()==original+suffix
    for pin in pins.values():assert sha(pin['path'])==pin['sha256']
    for pin in state['libraries'].values():assert sha(pin['path'])==pin['sha256']
    state.update(status='compiled_candidate_legacy_audio_lifecycle_library_and_probe',product_sha256=sha(exe),diagnostic_library_sha256=sha(rlib))
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(compile=str(out),profile=a.profile,product_sha256=state['product_sha256'],metadata_only=False,normal_product_source_modified=False),ensure_ascii=False))
