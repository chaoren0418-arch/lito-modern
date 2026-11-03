"""Run the owned asynchronous Presentation diagnostic after the G06 controller cohort."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):
    temp=p.with_name('.'+p.name+'.tmp');temp.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');os.replace(temp,p)
ap=argparse.ArgumentParser();ap.add_argument('--compile',type=Path,required=True);a=ap.parse_args();build=a.compile.resolve(strict=True)
assert build.is_relative_to(ROOT/'驗證紀錄/current/native1040-teleport-item-audio');b=read(build/'receipt.json');assert b['status']=='compiled_current_production_Presentation_test'
g06=Path(b['g06_run']);gc=read(g06/'cohort.json');assert gc['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases' and gc['owned_process'] is None
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and not queue.get('source_and_corresponding_QA_frozen'),'no audio runtime beside reserved current verification'
completion=read(g06/'completion-observation.json');assert completion['recorded_original_owned_processes_all_terminal'] and completion['source_manifest_sha256']==b['source_manifest_sha256']
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041
exe=Path(b['product']);probe=ROOT/'維護工具/candidates/native-teleport-item-audio/probe.rs'
fixture=gc['cross_endpoint_fixture_override'];harness=Path(fixture['path']);script=ROOT/'維護工具/run-teleport-handoff.ps1';validator=ROOT/'維護工具/verify-teleport-handoff.py'
jar=PROJECT/'target/current/server-reference/formal-server-C02AAAE8.jar';catalog=PROJECT/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
pins={str(p):sha(p) for p in [script,validator,harness,jar,catalog,build/'receipt.json',probe,Path(r'D:/lineage開發/0-客戶端/Sound/109.wav')]};assert pins[str(jar)]==gc['reference_jar_sha256'] and pins[str(catalog)]==gc['common_catalog_sha256'] and pins[str(harness)]==fixture['sha256']
assert sha(exe)==b['product_sha256'] and sha(probe)==sha(build/'probe.rs')==b['probe_sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native1040-teleport-item-audio'/('run-'+stamp);out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'probe.rs').write_bytes(probe.read_bytes())
assets=Path(r'D:/lineage開發/0-客戶端')
def metadata():return {str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in assets.iterdir() if p.is_file() and p.suffix.upper() in {'.PAK','.IDX'}}|{str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in [assets/'launcher/TW13081901.pak',assets/'Sound/717.wav',assets/'Sound/109.wav']}|{str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in (assets/'Sound').glob('*.wav') if p.is_file()}
before_assets=metadata();save(out/'asset-metadata-before.json',before_assets)
state=dict(status='prepared_owned_Presentation_handoff_diagnostic',profile=b['profile'],source_manifest_sha256=b['source_manifest_sha256'],compile=str(build),compile_receipt_sha256=sha(build/'receipt.json'),product=str(exe),product_sha256=sha(exe),prerequisite_G06=str(g06),pins=pins,owned_process=None,process_history=[],actual_sessions=0,normal_EXE_runtime_inferred=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
env={k:v for k,v in os.environ.items() if k not in {'JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'}};scratch=PROJECT/'target/current/scratch';env.update(TEMP=str(scratch),TMP=str(scratch))
env['JAVA_TOOL_OPTIONS']='-Dfile.encoding=UTF-8 -Dlito.resourceProfile=tw13081901-v1 -Dlito.extendedItemDetails=true -Dlito.commonCatalog='+str(catalog)+' -XX:-UsePerfData -Djava.io.tmpdir='+str(scratch)
state['JAVA_TOOL_OPTIONS_for_owned_children_only']=env['JAVA_TOOL_OPTIONS']
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetCurrentProcess.restype=wintypes.HANDLE
def created(h):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32);return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'runner-identity.json',dict(pid=os.getpid(),creation_utc=created(k.GetCurrentProcess())))
def stable():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in pins.items():assert sha(n)==h,n
    for v in b['libraries'].values():assert sha(v['path'])==v['sha256']
    assert sha(exe)==b['product_sha256']
def execute(command,label):
    d=out/label;d.mkdir()
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        j=dict(pid=child.pid,creation_utc=created(wintypes.HANDLE(int(child._handle))),command=command,label=label);state['process_history'].append(j);state['owned_process']=j;save(out/'receipt.json',state)
        j['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state)
    return j['exit_code']
family=ROOT/'驗證紀錄/current/native-teleport-handoff';before=set(family.iterdir()) if family.exists() else set()
print('PRESENTATION_RUN='+str(out),flush=True);save(out/'receipt.json',state)
try:
    stable();state['status']='executing_owned_Presentation_handoff';save(out/'receipt.json',state)
    code=execute(['pwsh','-NoProfile','-File',str(script),'-TestBinary',str(exe),'-ReferenceJar',str(jar),'-Scenario','cross-endpoint','-HarnessSource',str(harness)],'handoff')
    created_cases=set(family.iterdir())-before;assert len(created_cases)==1
    case=created_cases.pop();state['case']=str(case);save(out/'receipt.json',state)
    assert code==0,'preserve original diagnostic failure before repair'
    r=read(case/'receipt.json');assert r['inputs_unchanged'] and r['test_exit_code']==r['server_exit_code']==0 and r['test_binary_sha256']==b['product_sha256']
    assert execute(['python','-B','-X','utf8',str(validator),str(case)],'authority-comparison')==0
    runtime=Path(r['runtime']);reports={role:read(runtime/(role+'-handoff.json')) for role in ['a','b']}
    for role,v in reports.items():
        shutdown=read(runtime/(role+'-presentation-shutdown.json'));assert shutdown['client_quiescent'] and shutdown['Presentation_quiescent'] and shutdown['shutdown_error'] is None
        assert v['actual_Presentation_executed'] and v['destination_retirement']['view_quiescent'] and v['destination_retirement']['frame_absent'] and v['destination_retirement']['old_surface_rejected']
        assert not v['GUI_acceptance'] and not v['physical_audio_verified']
        retired=v['destination_retirement'];assert retired['retained_source_cues']==1 and retired['cue_outcomes']==['Stale'] and retired['repeated_cue_outcomes']==['AlreadyConsumed'] and retired['authentic_unplayed_source_cue_rejected']
        audio=v['destination_audio'];assert audio['own_deliveries_after']==audio['own_deliveries_before']+1
    state.update(status='executed_current_Presentation_handoff_diagnostic',runtime=str(runtime),actual_sessions=r['server']['actual_client_sessions'],peers=2,diagnostic_PNGs=4,view_workers_quiescent=True,authentic_unplayed_source_cues_rejected=2,new_scene_own_sound_events_delivered=2)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    stable();after_assets=metadata();save(out/'asset-metadata-after.json',after_assets);state['asset_metadata_unchanged']=before_assets==after_assets;assert before_assets==after_assets,'asset input changed; retain failure';save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],profile=b['profile'],actual_sessions=state['actual_sessions']),ensure_ascii=False),flush=True)
