"""Run current-source broad integration/main units in both profiles; preserve failure."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,sys,time
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def born(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
ap=argparse.ArgumentParser();ap.add_argument('--normal-run',type=Path,required=True);ap.add_argument('--G06-qualification',type=Path,required=True);a=ap.parse_args()
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
startup=a.normal_run.resolve(strict=True);assert startup.is_relative_to(ROOT/'驗證紀錄/current/native-installed-resource-profile-normal')
c=read(startup/'cohort.json');assert c['status']=='executed_installed_resource_profile_normal_related_both_profiles' and c['owned_process'] is None
normal=startup/'qualified/qualified.json';q=read(normal);source=q['source_manifest_sha256'];assert source==queue['current_product_identity']['source_manifest_sha256']
assert q['status']=='independently_qualified_installed_resource_profile_normal_related_both_profiles' and q['passed_per_profile']==19 and q['recorded_owned_processes_all_terminal']
assert q['observed_terminal_exit_code']==0 and not q['source_overlays_used'] and not q['full_library_verified']
inputs=read(startup/'compiled-inputs.json');assert len(inputs)==1049 and len(read(Path(c['application'])/'applied-source-inputs.json'))==1048
for profile in ['Debug','Release']:
    assert q['profiles'][profile]['normal']==c['profiles'][profile]['normal'] and q['normal_mains'][profile]==sha(c['profiles'][profile]['normal']['path'])

g06=a.G06_qualification.resolve(strict=True);assert g06.is_relative_to(ROOT/'驗證紀錄/current/native-installed-resource-profile-controller')
g=read(g06);assert g['actual_formal_sessions']==96 and g['source_manifest_sha256']==source and g['observed_terminal_exit_code']==0
names=['run-native-installed-resource-profile-default117.py','qualify-native-installed-resource-profile-default117.py','preserve-legacy-library-reports.py','native_installed_resource_profile_broad_gate.py','observe-native-installed-resource-profile-default117-completion.py',Path(__file__).name]
qa={n:sha(ROOT/'維護工具'/n) for n in names}
out=ROOT/'驗證紀錄/current/native-installed-resource-profile-default117'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n in names:(out/n).write_bytes((ROOT/'維護工具'/n).read_bytes())
state=dict(status='prepared_current_both_default117_profiles',output=str(out),pipeline_pid=os.getpid(),pipeline_creation_utc=born(k.GetCurrentProcess()),source_manifest_sha256=source,source_files=1048,compiled_input_files=1049,normal_qualification=str(normal),normal_qualification_sha256=sha(normal),G06_qualification=str(g06),G06_qualification_sha256=sha(g06),profiles={},validation_pins=qa,owned_process=None,process_history=[],source_and_corresponding_QA_frozen=True,full_library_in_this_scope=False,GUI_verified=False,whole_goal_complete=False)
persist=lambda:save(out/'cohort.json',state)
def check():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in qa.items():assert sha(ROOT/'維護工具'/n)==h,n
    assert sha(normal)==state['normal_qualification_sha256']
    for profile in ['Debug','Release']:
        assert sha(c['profiles'][profile]['normal']['path'])==q['normal_mains'][profile]

env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
def execute(args,folder):
    folder.mkdir();check()
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(args,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        record=dict(pid=p.pid,creation_utc=born(wintypes.HANDLE(int(p._handle))),arguments=args,output=str(folder),exit_code=None)
        state['process_history'].append(record);state['owned_process']=record;persist()
        code=p.wait();record['exit_code']=code;record['finished_utc']=datetime.now(timezone.utc).isoformat();state['owned_process']=None;persist()
    check();return code
persist();print('CURRENT1048_DEFAULT117_COHORT='+str(out),flush=True)
try:
    for profile in ['Debug','Release']:
        selected_normal=normal
        state.update(status='executing_current_default117_profile',active_profile=profile);persist()
        parent=ROOT/'驗證紀錄/current/default-native-regression';before=set(parent.iterdir())
        code=execute([sys.executable,'-B',str(ROOT/'維護工具/run-native-installed-resource-profile-default117.py'),'--profile',profile,'--normal-qualification',str(selected_normal),'--G06-qualification',str(g06)],out/(profile+'-console'))
        created=set(parent.iterdir())-before;assert len(created)==1;run=created.pop()
        state['profiles'][profile]=dict(run=str(run),actual_runner_exit_code=code,qualified=False);persist()
        assert code==0,'Preserve original broad failure; do not restart.'
        state['status']='independently_reading_current_default117_profile';persist()
        assert execute([sys.executable,'-B',str(ROOT/'維護工具/qualify-native-installed-resource-profile-default117.py'),str(run)],out/(profile+'-readback'))==0
        rq=run/'profile-qualified/qualified.json';row=read(rq);assert row['source_manifest_sha256']==source and row['actual_test_executables']==118 and row['counts']['failed']==0
        state['profiles'][profile].update(qualified=True,qualification=str(rq),qualification_sha256=sha(rq),counts=row['counts']);persist()
        print(profile+' default117 actual run and independent profile reader complete',flush=True)
    state.update(status='independently_qualified_current_default117_both_profiles',active_profile=None,source_and_corresponding_QA_frozen=False)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();persist()
print(json.dumps(dict(status=state['status'],output=str(out),counts={p:j['counts'] for p,j in state['profiles'].items()}),ensure_ascii=False),flush=True)
