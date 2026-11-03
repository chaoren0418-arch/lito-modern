"""Compile all four probes before twelve timed Presentation cases; preserve failure."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,sys,time
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';T=ROOT/'維護工具'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--g06-run',type=Path,required=True);a=ap.parse_args()
g06=a.g06_run.resolve(strict=True);assert g06.is_relative_to(ROOT/'驗證紀錄/current/native1034-G06-controller')
gc=read(g06/'cohort.json');go=read(g06/'completion-observation.json')
assert gc['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases' and gc['owned_process'] is None
assert go['recorded_original_owned_processes_all_terminal'] and go['actual_qualified_formal_sessions']==96
normal=Path(gc['startup_prerequisite']);inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035
tools=[T/(action+'-native1034-'+family+'.py') for action in ['compile','run','qualify'] for family in ['teleport-presentation','teleport-failure-presentation']]
tools += [T/'run-teleport-handoff.ps1',T/'run-teleport-failure.ps1',T/'verify-teleport-handoff.py',T/'verify-teleport-failure.py',T/'verify-native-current-teleport-cancel-expiry.py',T/'candidates/native-teleport-presentation/probe.rs',T/'candidates/native-teleport-failure-presentation/probe.rs',Path(gc['cross_endpoint_fixture_override']['path'])]
pins={str(p):sha(p) for p in tools}
out=ROOT/'驗證紀錄/current/native1034-presentation-cohort'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',inputs)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetCurrentProcess.restype=wintypes.HANDLE
def birth(h):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'runner-identity.json',dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())))
state=dict(status='prepared_presentation_cohort',source_manifest_sha256=gc['source_manifest_sha256'],G06=str(g06),normal_mains=gc['normal_mains'],tool_pins=pins,builds={},cases={},process_history=[],owned_process=None,actual_sessions=0,qualification_pending=True,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
env={n:v for n,v in os.environ.items() if n not in {'JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'}}
env.update(PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1',TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
def stable():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in pins.items():assert sha(n)==h,n
    c=read(normal/'cohort.json')
    for profile,h in state['normal_mains'].items():assert sha(c['profiles'][profile]['normal']['path'])==h
def execute(command,label,tag):
    d=out/label;d.mkdir();save(out/'cohort.json',state)
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=label)
        state['process_history'].append(item);state['owned_process']=item;save(out/'cohort.json',state)
        item['exit_code']=child.wait();state['owned_process']=None;save(out/'cohort.json',state)
    assert item['exit_code']==0,('preserve child failure',label)
    lines=[line[len(tag):] for line in (d/'stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith(tag)];assert len(lines)==1
    return Path(lines[0]).resolve(strict=True)
print('PRESENTATION_COHORT='+str(out),flush=True)
try:
    stable()
    for profile in ['Debug','Release']:
        for kind,family in [('success','teleport-presentation'),('failure','teleport-failure-presentation')]:
            state.update(status='compiling_presentation_probes',active_profile=profile,active_kind=kind)
            build=execute([sys.executable,'-B',str(T/('compile-native1034-'+family+'.py')),'--profile',profile,'--g06-run',str(g06)],profile+'-compile-'+kind,'PRESENTATION_COMPILE=')
            state['builds'][profile+'-'+kind]=str(build);stable()
    for profile in ['Debug','Release']:
        for scenario in ['cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-expiry']:
            stable();success=scenario=='cross-endpoint';kind='success' if success else 'failure';family='teleport-presentation' if success else 'teleport-failure-presentation'
            state.update(status='executing_presentation_case',active_profile=profile,active_scenario=scenario)
            command=[sys.executable,'-B',str(T/('run-native1034-'+family+'.py')),'--compile',state['builds'][profile+'-'+kind]]
            if not success:command+=['--scenario',scenario]
            run=execute(command,profile+'-'+scenario,'PRESENTATION_RUN=');receipt=read(run/'receipt.json')
            expected=6 if success else 10 if scenario=='wrong-service' else 8;assert receipt['actual_sessions']==expected
            state['cases'][profile+'-'+scenario]=dict(run=str(run),profile=profile,scenario=scenario,receipt_sha256=sha(run/'receipt.json'),actual_sessions=expected)
            state['actual_sessions']+=expected;save(out/'cohort.json',state)
            print(profile+' '+scenario+' completed; independent qualification pending',flush=True)
    assert len(state['cases'])==12 and state['actual_sessions']==96
    state.update(status='executed_presentation_cohort_qualification_pending',active_profile=None,active_scenario=None);stable()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],actual_sessions=state['actual_sessions'],whole_goal_complete=False),ensure_ascii=False),flush=True)
