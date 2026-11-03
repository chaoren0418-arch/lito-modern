"""Run remaining nonempty-audio branches with four already compiled probes."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,sys,time
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current';T=ROOT/'維護工具'
def read(p):
    for attempt in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):
    temp=Path(p).with_name('.'+Path(p).name+'.tmp');temp.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');os.replace(temp,p)
ap=argparse.ArgumentParser();ap.add_argument('--Debug-build',type=Path,required=True);ap.add_argument('--Release-build',type=Path,required=True);ap.add_argument('--retained-case',type=Path,required=True);a=ap.parse_args()
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and queue.get('source_and_corresponding_QA_frozen')
builds={profile:Path(getattr(a,profile+'_build')).resolve(strict=True) for profile in ['Debug','Release']}
receipts={profile:read(folder/'receipt.json') for profile,folder in builds.items()}
source=receipts['Debug']['source_manifest_sha256'];identity=receipts['Debug']['diagnostic_input_manifest_sha256']
inputs=read(builds['Debug']/'compiled-inputs.json');assert len(inputs)==1041
assert source==queue['current_product_identity']['source_manifest_sha256']
for profile,b in receipts.items():
    assert b['status']=='compiled_current_normal_body_audio_diagnostics' and b['owned_process'] is None and b['profile']==profile
    assert b['source_manifest_sha256']==source and b['diagnostic_input_manifest_sha256']==identity and b['normal_source_bodies_unchanged'] and not b['source_overlays_used']
    terminal=read(builds[profile]/'compile-terminal-observation.json');assert terminal['observed_terminal_exit_code']==0 and terminal['compile_receipt_sha256']==sha(builds[profile]/'receipt.json')
    assert read(builds[profile]/'compiled-inputs.json')==inputs
retained=a.retained_case.resolve(strict=True);assert retained.is_relative_to(V/'nativeCBC7-teleport-audio')
retained_qp=next(retained.glob('qualified-*/qualified.json'));rq=read(retained_qp);rr=read(retained/'receipt.json')
assert rq['status']=='independently_qualified_current_normal_bodies_nonempty_audio_branch' and rq['recorded_owned_all_terminal'] and rq['unplayed_source_audio_rejection_verified']
assert rq['source_manifest_sha256']==source and rq['diagnostic_input_manifest_sha256']==identity and rq['actual_sessions']==8
assert rq['profile'] in ['Debug','Release'] and rq['scenario']=='refused' and Path(rr['compile'])==builds[rq['profile']]
tools=[T/'run-nativeCBC7-teleport-audio.py',T/'qualify-nativeCBC7-teleport-audio.py',T/'run-teleport-handoff-75DA.ps1',T/'run-teleport-failure-75DA.ps1',T/'verify-teleport-handoff-75DA.py',T/'verify-teleport-failure-75DA.py',T/'verify-native-current-teleport-cancel-expiry-75DA.py']
pins={str(p):sha(p) for p in tools}
pins.update({str(folder/name):sha(folder/name) for folder in builds.values() for name in ['receipt.json','compile-terminal-observation.json','diagnostic-inputs.json']})
pins[str(retained_qp)]=sha(retained_qp)
out=V/'nativeCBC7-audio-cohort'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',inputs)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'runner-identity.json',dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())))
retained_key=rq['profile']+'-'+rq['scenario']
retained_record=dict(run=str(retained),profile=rq['profile'],scenario=rq['scenario'],actual_sessions=rq['actual_sessions'],
    receipt_sha256=sha(retained/'receipt.json'),retained_qualification=str(retained_qp),retained_qualification_sha256=sha(retained_qp),
    retained_original_session_id=rq['original_session_id'],retained_terminal_tool_chunk=rq['terminal_tool_chunk'],retained_without_rerun=True)
state=dict(status='prepared_current_normal_body_audio_cohort',source_manifest_sha256=source,diagnostic_input_manifest_sha256=identity,
    builds={p:str(d) for p,d in builds.items()},tool_pins=pins,cases={retained_key:retained_record},actual_sessions=8,
    process_history=[],owned_process=None,source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,
    normal_product_source_modified=False,normal_EXE_runtime_inferred=False,normal_GUI_API_permission_inferred=False,
    GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
env={key:value for key,value in os.environ.items() if key not in {'JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'}}
env.update(PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1',TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
def stable():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in pins.items():assert sha(n)==h,n
    for profile,b in receipts.items():
        assert sha(b['diagnostic_library'])==b['diagnostic_library_sha256']
        for product in b['products'].values():assert sha(product['path'])==product['sha256']
        normal=read(Path(b['normal_cohort'])/'cohort.json')
        for info in normal['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
def execute(command,label):
    d=out/label;d.mkdir()
    with(d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=label)
        state['process_history'].append(job);state['owned_process']=job;save(out/'cohort.json',state)
        job['exit_code']=child.wait();state['owned_process']=None;save(out/'cohort.json',state)
    assert job['exit_code']==0,('Preserve original branch failure',label)
    tag='NORMAL_AUDIO_RUN=';rows=[line[len(tag):] for line in(d/'stdout.log').read_text('utf-8-sig').splitlines() if line.startswith(tag)];assert len(rows)==1
    return Path(rows[0]).resolve(strict=True)
print('NORMAL_AUDIO_COHORT='+str(out),flush=True);save(out/'cohort.json',state)
try:
    stable()
    for profile in ['Debug','Release']:
        for scenario in ['refused','cross-endpoint','timeout','wrong-service','cancel-retry','cancel-expiry']:
            key=profile+'-'+scenario
            if key in state['cases']:continue
            stable();state.update(status='executing_current_normal_body_audio_branch',active_profile=profile,active_scenario=scenario)
            run=execute([sys.executable,'-B','-X','utf8',str(T/'run-nativeCBC7-teleport-audio.py'),'--compile',str(builds[profile]),'--scenario',scenario,'--cohort',str(out)],key)
            rr=read(run/'receipt.json');expected=6 if scenario=='cross-endpoint' else 10 if scenario=='wrong-service' else 8
            assert rr['status']=='executed_current_normal_body_nonempty_audio_case' and rr['actual_sessions']==expected
            assert rr['authentic_unplayed_source_cues_rejected']==2 and rr['new_scene_own_sound_events_delivered']==2
            state['cases'][key]=dict(run=str(run),profile=profile,scenario=scenario,actual_sessions=expected,receipt_sha256=sha(run/'receipt.json'))
            state['actual_sessions']+=expected;save(out/'cohort.json',state);print(key+' real nonempty audio branch completed; independent qualification pending',flush=True)
    assert len(state['cases'])==12 and len(state['process_history'])==11 and state['actual_sessions']==96
    state.update(status='executed_current_normal_body_audio_cohort_qualification_pending',active_profile=None,active_scenario=None);stable()
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'cohort.json',state)
print(json.dumps(dict(cohort=str(out),status=state['status'],actual_sessions=state['actual_sessions'],retained_cases_without_rerun=1),ensure_ascii=False))
