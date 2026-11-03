"""Compile one separate public-API probe after current normal-product/affected-test terminal gates."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,struct,subprocess,time,tomllib
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],required=True);ap.add_argument('--normal-run',type=Path,required=True);a=ap.parse_args()
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
normal=a.normal_run.resolve(strict=True);assert normal.is_relative_to(V/'native-same-endpoint-catalog-normal')
c=read(normal/'cohort.json');qp=normal/'qualified/qualified.json';q=read(qp);inputs=read(normal/'compiled-inputs.json');source=c['source_manifest_sha256']
assert c['status']=='executed_same_endpoint_catalog_normal_related_both_profiles'and c['owned_process']is None
assert q['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles'and q['source_manifest_sha256']==source and q['cohort_sha256']==sha(normal/'cohort.json')
assert q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_all_terminal']and q['passed_per_profile']==16 and q['actual_sessions']==0
assert not q['source_overlays_used']and not q['diagnostic_API_extension']and q['normal_product_build']
assert source==queue['current_product_identity']['source_manifest_sha256']==sha(normal/'source-inputs.json')and len(inputs)==1048 and len(read(normal/'source-inputs.json'))==1047
assert all(q['normal_mains'][p]==sha(item['normal']['path'])for p,item in c['profiles'].items())
candidate=ROOT/'維護工具/candidates/native-same-endpoint-catalog-probe';prep=read(candidate/'preparation.json');prep_sha=sha(candidate/'preparation.json')
assert prep['base_source_manifest_sha256']==source and sha(candidate/'probe.rs')==prep['probe_sha256']and not prep['compiled']and not prep['runtime_overrides']
libraries={};native=set()
for line in(normal/a.profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for name in row['filenames']:
            if name.endswith('.rlib'):
                key=row['target']['name'].replace('-','_')
                if key not in libraries or '/x86_64-pc-windows-msvc/'in name.replace('\\','/'):libraries[key]=name
assert {'lito_modern','anyhow','serde_json','png'}<=libraries.keys()
pins={name:dict(path=path,sha256=sha(path))for name,path in libraries.items()}
release=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['profile']['release'];assert tuple(release[k]for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
assert not {k:v for k,v in os.environ.items()if v and(k.startswith('CARGO_PROFILE_')or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
def guard():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for item in pins.values():assert sha(item['path'])==item['sha256']
    for item in c['profiles'].values():assert sha(item['normal']['path'])==item['normal']['sha256']
    assert sha(candidate/'preparation.json')==prep_sha and sha(candidate/'probe.rs')==prep['probe_sha256']
guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()
out=V/'native-catalog-handoff'/('compile-'+stamp);out.mkdir(parents=True);work=P/'target/current/native-catalog-handoff'/stamp;work.mkdir(parents=True)
for name in ['probe.rs','preparation.json']:(out/name).write_bytes((candidate/name).read_bytes())
(out/'compiler.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',inputs)
exe=work/'same-endpoint-rental.exe';rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
command=[str(rust),str(out/'probe.rs'),'--edition=2021','--crate-name','native_same_endpoint_rental_probe','--test','--target','x86_64-pc-windows-msvc','-o',str(exe)]
for path in [P/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps'),P/('target/current/'+a.profile.lower()+'/deps')]:command+=['-L','dependency='+str(path)]
for name,path in sorted(libraries.items()):command+=['--extern',name+'='+path]
for path in sorted(native):command+=['-L',path]
if a.profile=='Release':
    for key in ['opt-level','lto','codegen-units','strip']:command+=['-C',key+'='+str(release[key])]
else:command+=['-C','opt-level=0','-C','debuginfo=1','-C','debug-assertions=yes','-C','overflow-checks=yes']
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='compiling_normal_public_API_same_endpoint_rental_probe',profile=a.profile,source_manifest_sha256=source,source_files=1047,compiled_input_files=1048,normal_related_scope_only=True,full_current_library_verified=False,normal_cohort=str(normal),normal_qualification=str(qp),normal_qualification_sha256=sha(qp),probe_sha256=sha(out/'probe.rs'),candidate_preparation_sha256=prep_sha,libraries=pins,release_configuration=release,product=str(exe),runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),owned_process=None,process_history=[],runtime_overrides=False,source_overlays_used=False,normal_source_bodies_unchanged=True,normal_source_modified=False,diagnostic_API_extension=False,normal_product_build=False,actual_sessions=0,GUI_verified=False,whole_goal_complete=False)
save(out/'receipt.json',state);print('SAME_ENDPOINT_RENTAL_COMPILE='+str(out),flush=True)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
try:
    with(out/'stdout.log').open('wb')as stdout,(out/'stderr.log').open('wb')as stderr:
        child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command);state['owned_process']=job;state['process_history'].append(job);save(out/'receipt.json',state)
        try:job['exit_code']=child.wait()
        except BaseException:
            if child.poll()is None:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];job['cleanup_stopped_original_owned_handle']=True;child.terminate();child.wait(timeout=3)
            job['exit_code']=child.returncode;raise
        finally:state['owned_process']=None;save(out/'receipt.json',state)
    assert job['exit_code']==0,'preserve compiler diagnostics before correction'
    with exe.open('rb')as f:
        assert f.read(2)==b'MZ';f.seek(0x3c);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0'and struct.unpack('<H',f.read(2))[0]==0x8664
    state.update(status='compiled_normal_public_API_same_endpoint_rental_probe',product_sha256=sha(exe))
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:guard();save(out/'receipt.json',state)
print(json.dumps(dict(compile=str(out),profile=a.profile,status=state['status'],actual_sessions=0),ensure_ascii=False))
