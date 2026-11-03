"""Six sequential current-source cold C02 cases after all four probes compile."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,sys,time

ROOT=Path(__file__).resolve().parents[1]
P=ROOT/'LiTo381-Modern'
T=ROOT/'維護工具'
def read(path):
    for attempt in range(10):
        try:return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
parser=argparse.ArgumentParser()
for profile in ['debug','release']:
    for probe in ['live','moving']:parser.add_argument('--'+profile+'-'+probe,type=Path,required=True)
args=parser.parse_args()
builds={profile+'-'+probe:getattr(args,profile.lower()+'_'+probe).resolve(strict=True)for profile in ['Debug','Release']for probe in ['live','moving']}
receipts={name:read(path/'receipt.json')for name,path in builds.items()}
source={row['source_manifest_sha256']for row in receipts.values()}
assert source=={'4E1F7A4E02DA1201F2254A959A4130D5038C6F0A9393449D4EB30F8AE2883ED4'}
for name,row in receipts.items():
    assert builds[name].is_relative_to(ROOT/'驗證紀錄/current/native1040-cone-cold-scene')
    assert row['status']=='compiled_current1040_cold_source_diagnostic' and row['compiler']['exit_code']==0
    assert row['profile']+'-'+row['probe_kind']==name and row['normal_sources_body_compiled'] and not row['source_overrides_used']
inputs=read(builds['Debug-live']/'compiled-inputs.json')
assert len(inputs)==1041 and all(read(path/'compiled-inputs.json')==inputs for path in builds.values())
queue=read(ROOT/'工作佇列.json')
assert not queue.get('active_workloads') and not queue.get('source_and_corresponding_QA_frozen')
tools=[Path(__file__),T/'run-native1040-cone-cold-live.py',T/'qualify-native1040-cone-cold-live.py',T/'qualify-native1040-cone-cold-moving.py']
pins={str(path):sha(path)for path in tools}
out=ROOT/'驗證紀錄/current/native1040-cone-cold-cohort'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
save(out/'compiled-inputs.json',inputs)
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    values=[wintypes.FILETIME()for _ in range(4)]
    assert k.GetProcessTimes(handle,*[ctypes.byref(value)for value in values])
    ticks=values[0].dwLowDateTime|(values[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='prepared_current_cold_C02_cohort',source_manifest_sha256=next(iter(source)),runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),builds={name:str(path)for name,path in builds.items()},build_receipt_pins={name:sha(path/'receipt.json')for name,path in builds.items()},tool_pins=pins,cases={},process_history=[],owned_process=None,actual_sessions=0,actual_requests=0,actual_accepted_casts=0,qualification_pending=True,normal_sources_body_compiled=True,source_overrides_used=False,normal_EXE_runtime_inferred=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'runner-identity.json',state['runner'])
env=dict(os.environ,PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1',TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
def stable():
    for name,digest in inputs.items():assert sha(ROOT/name)==digest,name
    for name,digest in pins.items():assert sha(name)==digest,name
    for name,path in builds.items():
        row=receipts[name];assert sha(path/'receipt.json')==state['build_receipt_pins'][name]
        assert sha(row['product'])==row['product_sha256'] and sha(row['normal_product'])==row['normal_product_sha256']
        for pin in row['candidate_inputs'].values():assert sha(pin['path'])==pin['sha256']
        for pin in row['libraries'].values():assert sha(pin['path'])==pin['sha256']
print('COLD_COHORT='+str(out),flush=True)
save(out/'cohort.json',state)
try:
    stable()
    for profile in ['Debug','Release']:
        for scenario in ['positive','refused','moving']:
            stable();key=profile+'-'+scenario;folder=out/key;folder.mkdir()
            state.update(status='executing_current_cold_C02_case',active_profile=profile,active_scenario=scenario)
            command=[sys.executable,'-B',str(T/'run-native1040-cone-cold-live.py'),'--profile','disintegrate-refused'if scenario=='refused'else'disintegrate','--maxcasts','2','--compile',str(builds[profile+('-moving'if scenario=='moving'else'-live')])]
            with (folder/'stdout.log').open('wb')as stdout,(folder/'stderr.log').open('wb')as stderr:
                child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
                job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=key)
                state['owned_process']=job;state['process_history'].append(job);save(out/'cohort.json',state)
                job['exit_code']=child.wait();state['owned_process']=None;save(out/'cohort.json',state)
            paths=[line[len('CLASS2_LIVE='):]for line in(folder/'stdout.log').read_text('utf-8-sig').splitlines()if line.startswith('CLASS2_LIVE=')]
            assert len(paths)==1,('preserve missing child output',key)
            run=Path(paths[0]);state['cases'][key]=dict(run=str(run),profile=profile,scenario=scenario,actual_runner_exit_code=job['exit_code'])
            save(out/'cohort.json',state)
            assert job['exit_code']==0,('preserve original child failure',key)
            receipt=read(run/'receipt.json');assert receipt['actual_sessions']==4 and receipt['all_started_processes_terminal']
            state['cases'][key]['receipt_sha256']=sha(run/'receipt.json')
            for name in ['actual_sessions','actual_requests','actual_accepted_casts']:
                field='actual_casts'if name=='actual_accepted_casts'else name
                state[name]+=receipt[field]
            save(out/'cohort.json',state);print(key+' executed; independent qualification pending',flush=True)
    stable();assert len(state['cases'])==6 and state['actual_sessions']==24 and state['actual_requests']==10 and state['actual_accepted_casts']==8
    state.update(status='executed_current_cold_C02_cohort_qualification_pending',active_profile=None,active_scenario=None)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],actual_sessions=state['actual_sessions'],actual_requests=state['actual_requests'],actual_accepted_casts=state['actual_accepted_casts'],whole_goal_complete=False),ensure_ascii=False),flush=True)
