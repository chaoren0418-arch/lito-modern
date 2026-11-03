"""Measure current normal Model startup without network, API extensions or source overlays."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--input',type=Path,required=True);a=ap.parse_args();input_path=a.input.resolve(strict=True);assert input_path.is_relative_to(V/'native8CC5-scene-cost')
normal=V/'native-legacy-audio-normal/validation-20261018-104256-823814';c=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
profile_input=read(input_path);assert profile_input['source_manifest_sha256']==c['source_manifest_sha256']
def guard():
    q=read(ROOT/'工作佇列.json');assert not q.get('active_workloads') and q['source_and_corresponding_QA_frozen']
    assert q['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
    for n,h in inputs.items():assert sha(ROOT/n)==h
    for info in c['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
    assert sha(profile_input['capture']['path'])==profile_input['capture']['sha256']
guard()
probe=ROOT/'維護工具/candidates/native8CC5-scene-cost/profile.rs';stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=V/'native8CC5-scene-cost'/('measurement-'+stamp);out.mkdir(parents=True);runtime=P/'target/current/native8CC5-scene-cost'/stamp;runtime.mkdir(parents=True)
(out/'profile.rs').write_bytes(probe.read_bytes());(out/'input.json').write_bytes(input_path.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
libraries={};native=set()
for line in(normal/'Debug/test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for path in row['filenames']:
            if path.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/' in path.replace('\\','/'):libraries[name]=path
libraries={name:libraries[name] for name in ['lito_modern','anyhow','serde_json']}
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';exe=runtime/'profile-normal-scene-cost.exe'
args=[str(rust),str(out/'profile.rs'),'--crate-name','profile_normal_scene_cost','--crate-type','bin','--edition=2021','--target','x86_64-pc-windows-msvc','-C','opt-level=0','-C','debuginfo=1','-C','debug-assertions=yes','-C','overflow-checks=yes','-o',str(exe)]
for name,path in libraries.items():args+=['--extern',name+'='+path]
for path in [P/'target/current/x86_64-pc-windows-msvc/debug/deps',P/'target/current/debug/deps']:args+=['-L','dependency='+str(path)]
for path in sorted(native):args+=['-L',path]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='compiling_resource_only_normal_Model_profile',source_manifest_sha256=c['source_manifest_sha256'],
    runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,
    normal_library=dict(path=libraries['lito_modern'],sha256=sha(libraries['lito_modern'])),libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},
    probe_sha256=sha(out/'profile.rs'),input_sha256=sha(out/'input.json'),product=str(exe),
    modes=['parallel','a','b'],normal_source_bodies_unchanged=True,source_overlays_used=False,diagnostic_API_extension=False,
    normal_product_source_modified=False,new_sessions=0,new_requests=0,normal_GUI_runtime_inferred=False,physical_audio_verified=False,gameplay_qualification_granted=False,G11_qualified=False,whole_goal_complete=False)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
def execute(command,label):
    guard()
    with(out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
        child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=command)
        state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
        try:job['exit_code']=child.wait(timeout=180)
        except subprocess.TimeoutExpired:
            assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];job['measurement_watchdog_expired']=True
            child.terminate();job['exit_code']=child.wait();save(out/'receipt.json',state);raise
        state['owned_process']=None;save(out/'receipt.json',state);assert job['exit_code']==0,('Preserve profiling failure',label)
print('SCENE_COST_MEASUREMENT='+str(out),flush=True);save(out/'receipt.json',state)
try:
    execute(args,'compile');state['product_sha256']=sha(exe)
    for mode in state['modes']:
        state.update(status='measuring_resource_only_normal_Model_costs',active_mode=mode);save(out/'receipt.json',state)
        folder=out/mode;folder.mkdir();execute([str(exe),str(out/'input.json'),str(folder),mode],mode)
        report=read(folder/'report.json');assert not report['gameplay_qualification_granted'] and report['network_sessions']==0
        summary=[{k:row[k] for k in ['role','actor_count','load_us','first_draw_us','first_frame_us','warm_draw_us','within_existing_8s','input_ready','movement_ready']} for row in report['results']]
        print(json.dumps(dict(mode=mode,measurements=summary,gameplay_qualification_granted=False),ensure_ascii=False),flush=True)
    guard();assert sha(probe)==state['probe_sha256'] and sha(input_path)==state['input_sha256']
    for pin in state['libraries'].values():assert sha(pin['path'])==pin['sha256']
    state.update(status='measured_unchanged_normal_Model_resource_costs_only',active_mode=None)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
files={p.relative_to(out).as_posix():sha(p) for p in out.rglob('*') if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in files.items())
save(out/'readback.json',dict(raw_files=files,archive_sha256=sha(out/'raw.zip')))
print(json.dumps(dict(measurement=str(out),status=state['status'],new_sessions=0,gameplay_qualification_granted=False),ensure_ascii=False))
