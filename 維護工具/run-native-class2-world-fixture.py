"""Compile the owned C02 harness and check initial repository rows, no sockets."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern';PREP=ROOT/'維護工具/candidates/native-class2-world'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
normal=ROOT/'驗證紀錄/current/native77-lawful/validation-20261017-195458-430865'
c=read(normal/'cohort.json')
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(ROOT)
assert c['owned_process'] is None
p=read(PREP/'preparation.json');harness=Path(p['candidate_harness']);assert sha(harness)==p['candidate_harness_sha256']
jar=PROJECT/'target/current/server-reference/formal-server-C02AAAE8.jar';assert sha(jar)==p['reference_jar_sha256']
catalog=PROJECT/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin';assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1031
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-class2-world-fixture'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-class2-world-fixture'/stamp;runtime.mkdir(parents=True);assert runtime.resolve().is_relative_to(ROOT)
classes=runtime/'classes';classes.mkdir();(out/'NativeSelectorHarness.java').write_bytes(harness.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
state=dict(status='prepared_native77_repository_preflight',output=str(out),runtime=str(runtime),classes=str(classes),source_manifest_sha256=c['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,
    preparation=p,reference_jar=str(jar),reference_jar_sha256=sha(jar),resource_profile='tw13081901-v1',extended_item_details=True,common_catalog=dict(path=str(catalog),sha256=sha(catalog)),owned_process=None,process_history=[],profiles={},game_server_started=False,repository_only=True,new_sessions=0,new_casts=0,production_source_modified=False,whole_goal_complete=False)
env={k:v for k,v in os.environ.items() if k not in {'JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'}};env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'));jdk=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def check():
    assert sha(harness)==p['candidate_harness_sha256'] and sha(jar)==p['reference_jar_sha256']
    assert sha(catalog)==state['common_catalog']['sha256']
    for n,h in inputs.items():assert sha(ROOT/n)==h
def execute(command,label,cwd):
    check();d=out/label;d.mkdir()
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=cwd,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];assert kernel.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in times]);ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        job=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=command,label=label);state['owned_process']=job;state['process_history'].append(job);save(out/'receipt.json',state)
        job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state)
    check();assert job['exit_code']==0,label+' failed; preserve before repair'
print('NATIVE77_FIXTURE_PREFLIGHT='+str(out),flush=True)
try:
    execute([str(jdk/'javac.exe'),'-J-XX:-UsePerfData','-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/'NativeSelectorHarness.java')],'compile',runtime)
    state['class_pins']={str(f):sha(f) for f in classes.rglob('*.class')};assert len(state['class_pins'])==2;save(out/'receipt.json',state)
    for profile in p['profiles']:
        case=runtime/profile;case.mkdir();assert case.resolve().is_relative_to(ROOT)
        execute([str(jdk/'java.exe'),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+str(PROJECT/'target/current/scratch'),'-Duser.dir='+str(case),'-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',str(classes)+os.pathsep+str(jar),'local.lito381.server.NativeSelectorHarness',str(case),profile,'--verify-only'],profile,case)
        report=case/'initial-disintegrate-fixture.json';data=read(report);assert data['profile']==profile and data['before_all_sessions'] and data['runtime_mutations']==0 and data['natural_special_critical']
        mage=next(row for row in data['rows'] if row['object_id']==3);assert mage['learned77'] and mage['max_hp']==32767 and mage['max_mp']==(2 if profile=='disintegrate-refused' else 5000) and mage['lawful']==(499 if profile=='disintegrate-lawful' else 1000)
        (out/(profile+'.json')).write_bytes(report.read_bytes());state['profiles'][profile]=dict(report=str(report),sha256=sha(report),rows=data['rows']);save(out/'receipt.json',state)
    state.update(status='executed_native77_initial_repository_profiles_only',fixture_profiles_passed=1)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],fixture_profiles_passed=1,new_sessions=0,new_casts=0),ensure_ascii=False),flush=True)
