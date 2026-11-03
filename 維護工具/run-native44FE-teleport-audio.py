"""Run one pinned real 75DA branch with a nonempty scoped ordinary source cue."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time
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
ap=argparse.ArgumentParser();ap.add_argument('--compile',type=Path,required=True);ap.add_argument('--scenario',choices=['cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-expiry'],required=True);ap.add_argument('--cohort',type=Path);a=ap.parse_args()
build=a.compile.resolve(strict=True);assert build.is_relative_to(V/'native44FE-teleport-audio')
b=read(build/'receipt.json');assert b['status']=='compiled_current_normal_body_audio_diagnostics'
ct=read(build/'compile-terminal-observation.json');assert ct['observed_terminal_exit_code']==0 and ct['compile_receipt_sha256']==sha(build/'receipt.json')
queue=read(ROOT/'工作佇列.json');assert queue.get('source_and_corresponding_QA_frozen')
assert queue['current_product_identity']['source_manifest_sha256']==b['source_manifest_sha256']
if queue.get('active_workloads'):
    assert a.cohort and set(queue['active_workloads'])=={'G06_audio'}
    reservation=queue['active_workloads']['G06_audio'];assert Path(reservation['run']).resolve(strict=True)==a.cohort.resolve(strict=True)
    assert reservation['source_manifest_sha256']==b['source_manifest_sha256']
g06=Path(b['g06_run']);gc=read(g06/'cohort.json');go=read(g06/'completion-observation.json')
assert gc['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases' and gc['owned_process'] is None
assert go['recorded_original_owned_processes_all_terminal'] and go['source_manifest_sha256']==b['source_manifest_sha256']
kind='success' if a.scenario=='cross-endpoint' else 'failure';exe=Path(b['products'][kind]['path']);probe=Path(b['candidate_inputs'][kind+'.rs']['path'])
assert probe.is_relative_to(T/'candidates') and sha(probe)==sha(build/(kind+'.rs'))==b['candidate_inputs'][kind+'.rs']['sha256']
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041
normal=read(Path(b['normal_cohort'])/'cohort.json')
if kind=='success':
    fixture=gc['cross_endpoint_fixture_override'];harness=Path(fixture['path']);harness_sha=fixture['sha256']
    script=T/'run-teleport-handoff-75DA.ps1';validator=T/'verify-teleport-handoff-75DA.py';case_family=V/'native-teleport-handoff'
else:
    harness=P/'tests/oracle/NativeTeleportFailureHarness.java';harness_sha=inputs[harness.relative_to(ROOT).as_posix()]
    script=T/'run-teleport-failure-75DA.ps1';validator=T/('verify-native-current-teleport-cancel-expiry-75DA.py' if a.scenario=='cancel-expiry' else 'verify-teleport-failure-75DA.py');case_family=V/'native-teleport-failure'
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assets=Path(r'D:/lineage開發/0-客戶端')
pin_paths=[script,validator,harness,jar,catalog,build/'receipt.json',build/'compile-terminal-observation.json',probe,Path(b['formal_C1F_binding']['path'])]+[assets/('Sound/'+str(sound)+'.wav') for sound in [109,1534,1564]]
pins={str(p):sha(p) for p in pin_paths}
assert sha(formal)==pins[str(jar)]==gc['reference_jar_sha256'] and pins[str(catalog)]==gc['common_catalog_sha256'] and pins[str(harness)]==harness_sha
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native44FE-teleport-audio'/('run-'+stamp);out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'probe.rs').write_bytes(probe.read_bytes())
def metadata():return {str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in assets.iterdir() if p.is_file() and p.suffix.upper() in {'.PAK','.IDX'}}|{str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in [assets/'launcher/TW13081901.pak']}|{str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in (assets/'Sound').glob('*.wav') if p.is_file()}
before_assets=metadata();save(out/'asset-metadata-before.json',before_assets)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetCurrentProcess.restype=wintypes.HANDLE
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'runner-identity.json',dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())))
state=dict(status='prepared_current_normal_body_nonempty_audio_case',profile=b['profile'],scenario=a.scenario,kind=kind,
    source_manifest_sha256=b['source_manifest_sha256'],diagnostic_input_manifest_sha256=b['diagnostic_input_manifest_sha256'],
    compile=str(build),compile_receipt_sha256=sha(build/'receipt.json'),product=str(exe),product_sha256=b['products'][kind]['sha256'],
    prerequisite_G06=str(g06),pins=pins,owned_process=None,process_history=[],actual_sessions=0,
    source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,
    normal_GUI_API_permission_inferred=False,normal_product_source_modified=False,normal_EXE_runtime_inferred=False,
    GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
env={key:value for key,value in os.environ.items() if key not in {'JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'}}
scratch=P/'target/current/scratch';env.update(TEMP=str(scratch),TMP=str(scratch),PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1')
env['JAVA_TOOL_OPTIONS']='-Dfile.encoding=UTF-8 -Dlito.resourceProfile=tw13081901-v1 -Dlito.extendedItemDetails=true -Dlito.commonCatalog='+str(catalog)+' -XX:-UsePerfData -Djava.io.tmpdir='+str(scratch)
state['JAVA_TOOL_OPTIONS_for_owned_children_only']=env['JAVA_TOOL_OPTIONS']
def stable():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in pins.items():assert sha(n)==h,n
    for pin in b['libraries'].values():assert sha(pin['path'])==pin['sha256']
    assert sha(formal)==gc['reference_jar_sha256'] and sha(b['diagnostic_library'])==b['diagnostic_library_sha256'] and sha(exe)==state['product_sha256']
    for info in normal['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
def execute(command,label):
    d=out/label;d.mkdir()
    with(d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=label)
        state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
        job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state)
    return job['exit_code']
before_cases=set(case_family.iterdir()) if case_family.exists() else set()
print('NORMAL_AUDIO_RUN='+str(out),flush=True);save(out/'receipt.json',state)
try:
    stable();state['status']='executing_current_normal_body_nonempty_audio_case';save(out/'receipt.json',state)
    command=['pwsh','-NoProfile','-File',str(script),'-TestBinary',str(exe),'-ReferenceJar',str(jar),'-Scenario',a.scenario]
    if kind=='success':command+=['-HarnessSource',str(harness)]
    code=execute(command,'handoff');created=set(case_family.iterdir())-before_cases;assert len(created)==1,created
    case=created.pop();state['case']=str(case);save(out/'receipt.json',state);assert code==0,'Preserve original diagnostic failure'
    cr=read(case/'receipt.json');assert cr['inputs_unchanged'] and cr['test_exit_code']==cr['server_exit_code']==0 and cr['test_binary_sha256']==state['product_sha256']
    assert execute(['python','-B','-X','utf8',str(validator),str(case)],'authority-comparison')==0
    runtime=Path(cr['runtime'])
    for role in ['a','b']:
        v=read(runtime/(role+('-handoff.json' if kind=='success' else '-failure.json')))
        shutdown=read(runtime/(role+'-presentation-shutdown.json'));assert shutdown['client_quiescent'] and shutdown['Presentation_quiescent'] and shutdown['shutdown_error'] is None
        retired=v['destination_retirement' if kind=='success' else 'failure_retirement']
        assert v['actual_Presentation_executed'] and retired['view_quiescent'] and retired['frame_absent'] and retired['old_surface_rejected']
        assert retired['retained_source_cues']==1 and retired['audio_deliveries_before']==retired['audio_deliveries_after']
        if kind=='success':
            assert retired['cue_outcomes']==['Stale'] and retired['repeated_cue_outcomes']==['AlreadyConsumed']
            assert v['destination_audio']['own_deliveries_after']==v['destination_audio']['own_deliveries_before']+1
        else:
            assert retired['cue_outcomes']==[dict(first='Stale',repeat='AlreadyConsumed')]
            recovery=v['recovery_audio'];assert recovery['first'].startswith('Played(Started') and recovery['repeat']=='AlreadyConsumed' and recovery['deliveries_after']==recovery['deliveries_before']+1
        source=v['source_audio'];own=source['before']['world']['status']['object_id'];graphics=source['before']['world']['actors'][str(own)]['graphics']
        assert graphics in [0,61] and source['request_hex']=='1f44' and source['sound']=={0:1534,61:1564}[graphics] and source['actual_action_request']
        assert not v['GUI_acceptance'] and not v['physical_audio_verified']
    expected=6 if kind=='success' else 10 if a.scenario=='wrong-service' else 8
    assert cr['server']['actual_client_sessions']==expected
    state.update(status='executed_current_normal_body_nonempty_audio_case',runtime=str(runtime),actual_sessions=expected,
        peers=2,diagnostic_PNGs=4,view_workers_quiescent=True,authentic_unplayed_source_cues_rejected=2,new_scene_own_sound_events_delivered=2)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    stable();after_assets=metadata();save(out/'asset-metadata-after.json',after_assets)
    state['asset_metadata_unchanged']=before_assets==after_assets;save(out/'receipt.json',state);assert before_assets==after_assets
print(json.dumps(dict(output=str(out),status=state['status'],profile=b['profile'],scenario=a.scenario,actual_sessions=state['actual_sessions']),ensure_ascii=False),flush=True)
