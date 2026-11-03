"""Observe current same-endpoint behavior using existing exact-source products; never deploy a JAR."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current';T=ROOT/'維護工具'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],required=True);a=ap.parse_args();profile=a.profile
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads')and queue['source_and_corresponding_QA_frozen']
base=V/'native1040-G06-controller/20261018-161827-160049';c=read(base/'cohort.json');q=read(base/'qualified/qualified.json');o=read(base/'completion-observation.json');assert q['actual_formal_sessions']==96 and o['recorded_original_owned_processes_all_terminal']and o['observed_terminal_exit_code']==0
assert c['source_manifest_sha256']==queue['current_product_identity']['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
pins=read(base/'source-inputs.json');products=read(base/profile/'products.json');exe=Path(products['native_teleport_handoff']['path']);jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';harness=T/'candidates/native-teleport-fixture-arm/NativeTeleportHarness.java';script=T/'run-teleport-handoff-75DA.ps1';catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
extra={str(p):sha(p)for p in[exe,jar,formal,harness,script,catalog]};assert extra[str(exe)]==products['native_teleport_handoff']['sha256']and extra[str(jar)]==extra[str(formal)]==c['reference_jar_sha256']and extra[str(harness)]==c['cross_endpoint_fixture_override']['sha256']
def guard():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for n,h in extra.items():assert sha(Path(n))==h,n
    for profile,info in read(Path(c['startup_prerequisite'])/'cohort.json')['profiles'].items():assert sha(Path(info['normal']['path']))==c['normal_mains'][profile]
guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+profile.lower();out=V/'native44FE-same-endpoint'/stamp;out.mkdir(parents=True);(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'source-inputs.json').write_bytes((base/'source-inputs.json').read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32);return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
command=['pwsh','-NoProfile','-File',str(script),'-TestBinary',str(exe),'-ReferenceJar',str(jar),'-Scenario','same-endpoint','-HarnessSource',str(harness)];env=dict(os.environ,PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1',TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'));env['JAVA_TOOL_OPTIONS']='-Dfile.encoding=UTF-8 -Dlito.resourceProfile=tw13081901-v1 -Dlito.extendedItemDetails=true -Dlito.commonCatalog='+str(catalog)+' -XX:-UsePerfData -Djava.io.tmpdir='+str(P/'target/current/scratch')
state=dict(status='running_current_same_endpoint_observation',profile=profile,source_manifest_sha256=c['source_manifest_sha256'],reference_jar_sha256=c['reference_jar_sha256'],base_controller=str(base),normal_mains=c['normal_mains'],pins=extra,runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,formal_deployed=False,normal_source_modified=False,gameplay_qualification_granted=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
family=V/'native-teleport-handoff';before=set(family.iterdir());print('SAME_ENDPOINT_RUN='+str(out),flush=True)
try:
    with(out/'stdout.log').open('wb')as stdout,(out/'stderr.log').open('wb')as stderr:
        child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label='same-endpoint');state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state);job['exit_code']=child.wait();state['owned_process']=None
    created=set(family.iterdir())-before;assert len(created)==1;case=created.pop();receipt=read(case/'receipt.json');assert receipt['scenario']=='same-endpoint'and receipt['jar_sha256']==c['reference_jar_sha256']and receipt['test_binary_sha256']==products['native_teleport_handoff']['sha256'];guard()
    state.update(status='current_same_endpoint_run_terminal_pending_independent_readback',case=str(case),actual_PowerShell_exit_code=job['exit_code'],Rust_exit_code=receipt.get('test_exit_code'),server_exit_code=receipt.get('server_exit_code'),case_receipt_sha256=sha(case/'receipt.json'))
except BaseException as error:state.update(status='observation_driver_failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(run=str(out),case=state['case'],PowerShell_exit_code=state['actual_PowerShell_exit_code'],Rust_exit_code=state['Rust_exit_code'],gameplay_qualification_granted=False),ensure_ascii=False))
