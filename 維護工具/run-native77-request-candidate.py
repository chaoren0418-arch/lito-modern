"""Read-only source-pinned C14 candidate unit probe, no live controller/network."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern';PREP=ROOT/'維護工具/candidates/native77-request'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
p=read(PREP/'preparation.json');normal=ROOT/'驗證紀錄/current/native77-scene-integration/validation-20261017-183727-278843'
qp=normal/'Debug-qualified-20261017-184201-581264/qualified.json';q=read(qp);assert sha(qp)=='577E53AF8817257525800EA14028EAF4E1E834C4AF04AFB90FA3551A9AF737CE'
assert q['source_manifest_sha256']==p['baseline_source_manifest_sha256'];inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1031
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,h in p['before'].items():assert sha(PROJECT/'src/modern_lito'/n)==h
for n,h in p['after'].items():assert sha(PREP/n)==h
locator=ROOT/'驗證紀錄/current/default-native-regression/20261017-135100-713-Debug/build/stdout.log';libraries={};native=set()
for line in locator.read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='compiler-artifact' and not row['profile']['test'] and row['target']['name'] in {'lito_modern','anyhow','serde_json','serde'}:
        paths=[x for x in row['filenames'] if x.endswith('.rlib')]
        if paths:assert len(paths)==1;libraries[row['target']['name']]=dict(path=paths[0],sha256=sha(paths[0]))
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
assert set(libraries)=={'lito_modern','anyhow','serde_json','serde'}
first=read(ROOT/'驗證紀錄/current/native77-scene-integration/validation-20261017-183221-124368/cohort.json')
mtime=datetime.fromtimestamp(Path(libraries['lito_modern']['path']).stat().st_mtime,tz=timezone.utc)
start=next(j['creation_utc'] for j in first['process_history'] if j['label']=='Debug/normal-build');end=next(j['creation_utc'] for j in first['process_history'] if j['label']=='Debug/version')
assert datetime.fromisoformat(start)<=mtime<=datetime.fromisoformat(end),'borrow only the normal current-source Debug library'
current=read(normal/'cohort.json');assert current['owned_process']['label'].endswith(('normal-build','lib-build','integration-build')),'do not compile beside timed runtime cases'
mains={profile:info['normal'] for profile,info in current['profiles'].items()}
oracle=ROOT/'驗證紀錄/current/native-attack-five-contract-inspection/20261017-141350-118886/tw13081901-v1.stdout.txt'
vectors=[json.loads(row) for row in oracle.read_text(encoding='utf-8-sig').splitlines() if json.loads(row)['skill']==77]
assert len(vectors)==2 and {v['request'] for v in vectors}=={'140904c800000002800180'}
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native77-request'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native77-request'/stamp;runtime.mkdir(parents=True)
probe=PREP/'request_probe.rs';pins={str(PREP/n):sha(PREP/n) for n in ['native_skills.rs','native_skill_selection.rs','request_probe.rs']}
wrapper=out/'probe.rs';wrapper.write_text(probe.read_text(encoding='utf-8-sig').replace('__SKILLS_PATH__',(PREP/'native_skills.rs').as_posix()).replace('__SELECTION_PATH__',(PREP/'native_skill_selection.rs').as_posix()).replace('__WORLD_FIXTURE__',(PROJECT/'tests/fixtures/native-world-current.json').as_posix()),encoding='utf-8',newline='\n')
for name in ['native_skills.rs','native_skill_selection.rs','request_probe.rs']:(out/name).write_bytes((PREP/name).read_bytes())
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
state=dict(status='prepared_isolated_native77_request',source_manifest_sha256=q['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,normal_qualification=str(qp),normal_qualification_sha256=sha(qp),
    libraries=libraries,library_emission_window=[start,end],candidate_pins=pins,preparation=p,normal_mains_before=mains,oracle=dict(path=str(oracle),sha256=sha(oracle),vectors=vectors),
    owned_process=None,process_history=[],production_source_modified=False,new_sessions=0,new_casts=0,full_ClientSession_executed=False,GUI_verified=False,whole_goal_complete=False)
env={k:v for k,v in os.environ.items() if not k.startswith('LITO_')};env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def check():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in pins.items():assert sha(n)==h
    for v in libraries.values():assert sha(v['path'])==v['sha256']
    for v in mains.values():assert sha(v['path'])==v['sha256']
def execute(command,label):
    check();d=out/label;d.mkdir()
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];assert kernel.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in times])
        ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32);job=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=command,label=label)
        state['owned_process']=job;state['process_history'].append(job);save(out/'receipt.json',state);job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state)
    check();assert job['exit_code']==0,label+' failed; preserve before repair'
print('NATIVE77_REQUEST_CANDIDATE='+str(out),flush=True)
try:
    exe=runtime/'native77-request-tests.exe';rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
    command=[str(rust),str(wrapper),'--edition=2021','--crate-name','native77_request_candidate','--test','--target','x86_64-pc-windows-msvc','-C','opt-level=0','-C','debuginfo=1','-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
    for name,v in libraries.items():command+=['--extern',name+'='+v['path']]
    for value in sorted(native):command+=['-L',value]
    execute(command,'compile');state['product']=dict(path=str(exe),sha256=sha(exe));save(out/'receipt.json',state)
    execute([str(exe),'--list','--format','terse'],'inventory');execute([str(exe),'--list','--ignored','--format','terse'],'ignored-inventory');execute([str(exe),'--test-threads=1'],'unit')
    state.update(status='executed_isolated_native77_request_candidate',passed=5,failed=0,ignored=0)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],production_source_modified=False,new_casts=0),ensure_ascii=False),flush=True)
