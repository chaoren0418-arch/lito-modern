"""Load selected normal actor wave/sound data only while the current queue is idle."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as s:return hashlib.file_digest(s,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
def guard():
    queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and not queue.get('source_and_corresponding_QA_frozen')
    q=queue['current_native1040_library'];assert q['source_manifest_sha256']==read(normal/'cohort.json')['source_manifest_sha256']
    assert q['recorded_original_owned_processes_all_terminal'] and q['observed_actual_exit_code']==0
    for n,h in inputs.items():assert sha(ROOT/n)==h
guard()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=V/'native-rental-action-audio-inspection'/stamp;out.mkdir(parents=True)
runtime=P/'target/current/native-rental-action-audio-inspection'/stamp;runtime.mkdir(parents=True)
probe=ROOT/'維護工具/candidates/native-rental-action-audio-inspection-b/probe.rs'
(out/'probe.rs').write_bytes(probe.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
captures=list((P/'target/current/native-teleport-handoff/20261018-100839-269/captures').glob('*/frames.jsonl'));assert len(captures)==1
capture=captures[0];libraries={};native=set()
for line in (normal/'Debug/test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for f in row['filenames']:
            if f.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/'in f.replace('\\','/'):libraries[name]=f
assert {'lito_modern','anyhow','serde_json','sha2'}<=libraries.keys()
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
exe=runtime/'inspect-source-wave.exe'
command=[str(rust),str(out/'probe.rs'),'--crate-name','inspect_source_wave','--edition=2021','--crate-type','bin','--target','x86_64-pc-windows-msvc','-o',str(exe)]
for name,path in sorted(libraries.items()):command+=['--extern',name+'='+path]
for directory in [P/'target/current/x86_64-pc-windows-msvc/debug/deps',P/'target/current/debug/deps']:command+=['-L','dependency='+str(directory)]
for directory in sorted(native):command+=['-L',directory]
env=os.environ.copy();env.update(CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def born(handle):
    times=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in times]);ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='compiling_selected_normal_actor_data_probe',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],
    runner_identity=dict(pid=os.getpid(),creation_utc=born(k.GetCurrentProcess())),normal_run=str(normal),
    libraries={n:dict(path=p,sha256=sha(p))for n,p in libraries.items()},capture=dict(path=str(capture),sha256=sha(capture)),
    probe_sha256=sha(probe),product=str(exe),normal_product_source_modified=False,new_sessions=0,new_casts=0,
    C_ACTION_sent=False,audio_delivery_executed=False,process_history=[],whole_goal_complete=False)
def execute(args,label):
    with (out/(label+'.stdout.txt')).open('wb')as stdout,(out/(label+'.stderr.txt')).open('wb')as stderr:
        child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=born(wintypes.HANDLE(int(child._handle))),arguments=args)
        state['process_history'].append(job);save(out/'receipt.json',state)
        job['exit_code']=child.wait();save(out/'receipt.json',state)
        assert job['exit_code']==0,label
print('INSPECTION_RUN='+str(out),flush=True)
try:
    execute(command,'compile');guard();state['product_sha256']=sha(exe)
    state['status']='loading_selected_normal_actor_wave_and_sound';save(out/'receipt.json',state)
    execute([str(exe),str(Path('D:/lineage開發/0-客戶端')),str(capture),str(out/'report.json')],'assets')
    guard();assert sha(capture)==state['capture']['sha256'] and sha(probe)==state['probe_sha256']
    for item in state['libraries'].values():assert sha(item['path'])==item['sha256']
    report=read(out/'report.json');assert report['action']==68 and report['captured_actor_graphics']==61
    assert report['events'] and not report['C_ACTION_sent'] and not report['audio_delivery_executed']
    state.update(status='executed_selected_normal_actor_wave_and_sound_data_only',report_sha256=sha(out/'report.json'))
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
pins={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as archive:
    for name in pins:archive.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip')as archive:assert all(hashlib.sha256(archive.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
print(json.dumps(dict(run=str(out),events=report['events'],sounds=report['sounds'],actual_sessions=0),ensure_ascii=False))
