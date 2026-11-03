"""Run each current integration/main profile once, then independently read it back."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,sys
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def creation(handle):
    times=[wintypes.FILETIME() for _ in range(4)];k=ctypes.WinDLL('kernel32',use_last_error=True)
    k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
    assert k.GetProcessTimes(handle,*[ctypes.byref(v) for v in times])
    ticks=(times[0].dwHighDateTime<<32)|times[0].dwLowDateTime
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
ap=argparse.ArgumentParser();ap.add_argument('--normal-qualification',type=Path,required=True);ap.add_argument('--G06-qualification',type=Path,required=True);ap.add_argument('--normal-kind',choices=['selector-related'],default='selector-related');a=ap.parse_args()
normal=a.normal_qualification.resolve(strict=True);g06=a.G06_qualification.resolve(strict=True)
assert g06.is_relative_to(ROOT/'驗證紀錄/current/native-G06-controller')
n=read(normal);g=read(g06);finished=read(g06.parent.parent/'completion-observation.json')
assert a.normal_kind=='selector-related' and normal.is_relative_to(ROOT/'驗證紀錄/current/native-selector-integration')
assert n['status']=='independently_qualified_current_normal_selector_both_profiles' and n['observed_terminal_exit_code']==0
assert n['recorded_owned_processes_all_terminal'] and n['recorded_owned_identities']==64
assert all(v['actual_test_executables']==29 and v['new_registered_selector_S39_tests_passed']==4 for v in n['profiles'].values())
normal_build=normal.parent
assert n['source_manifest_sha256']==g['source_manifest_sha256']
assert g['actual_formal_sessions']==96 and g['observed_terminal_exit_code']==0 and finished['recorded_original_owned_processes_all_terminal']
assert finished['qualification_sha256']==sha(g06)
pins=read(normal_build/'source-inputs.json');assert len(pins)==1028
out=ROOT/'驗證紀錄/current/native-default111'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
names=['run-native-selector-default111.py','qualify-native-selector-default111.py','preserve-legacy-library-reports.py',Path(__file__).name]
qa={name:sha(ROOT/'維護工具'/name) for name in names}
for name in names:(out/name).write_bytes((ROOT/'維護工具'/name).read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
state=dict(status='prepared_current_default111_both_profiles',output=str(out),pipeline_pid=os.getpid(),pipeline_creation_utc=creation(k.GetCurrentProcess()),normal_qualification=str(normal),normal_qualification_sha256=sha(normal),G06_qualification=str(g06),G06_qualification_sha256=sha(g06),source_files=1028,compiled_input_files=1029,source_manifest_sha256=n['source_manifest_sha256'],validation_pins=qa,profiles={},process_history=[],owned_process=None,source_and_corresponding_QA_frozen=True,GUI_verified=False,full_library_in_this_scope=False,whole_goal_complete=False)
state['normal_input_kind']=a.normal_kind
persist=lambda:save(out/'cohort.json',state)
def check():
    assert sha(normal)==state['normal_qualification_sha256'] and sha(g06)==state['G06_qualification_sha256']
    for name,h in pins.items():assert sha(ROOT/name)==h,name
    for name,h in qa.items():assert sha(ROOT/'維護工具'/name)==h,name
env=os.environ.copy();env.update(PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'))
def execute(command,folder):
    folder.mkdir();check()
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(command,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,creation_utc=creation(wintypes.HANDLE(int(p._handle))),arguments=command,output=str(folder),exit_code=None)
        state['owned_process']=item;state['process_history'].append(item);persist();waits=0
        while True:
            try:code=p.wait(timeout=30);break
            except subprocess.TimeoutExpired:
                waits+=1;persist()
                if waits%4==0:print(json.dumps(dict(original_child_live=p.pid,profile=state['active_profile'],step=state['status'])),flush=True)
    item['exit_code']=code;item['finished_utc']=datetime.now(timezone.utc).isoformat();state['owned_process']=None;persist();check();return code
persist();print('DEFAULT111_COHORT='+str(out),flush=True)
try:
    for profile in ['Debug','Release']:
        state.update(status='executing_current_default111_profile',active_profile=profile);persist()
        parent=ROOT/'驗證紀錄/current/default-native-regression';before=set(parent.iterdir())
        code=execute([sys.executable,'-X','utf8',str(ROOT/'維護工具/run-native-selector-default111.py'),'--profile',profile,'--normal-qualification',str(normal),'--G06-qualification',str(g06),'--normal-kind',a.normal_kind],out/(profile+'-console'))
        created=set(parent.iterdir())-before;assert len(created)==1
        run=created.pop();state['profiles'][profile]=dict(run=str(run),actual_runner_exit_code=code,qualified=False);persist()
        assert code==0,'Preserve actual profile failure; do not restart.'
        state['status']='independently_reading_current_default111_profile';persist()
        assert execute([sys.executable,'-X','utf8',str(ROOT/'維護工具/qualify-native-selector-default111.py'),str(run)],out/(profile+'-readback'))==0
        qualified=run/'profile-qualified/qualified.json';q=read(qualified)
        assert q['source_manifest_sha256']==state['source_manifest_sha256'] and q['actual_test_executables']==112 and q['counts']['failed']==0
        state['profiles'][profile].update(qualified=True,qualification=str(qualified),qualification_sha256=sha(qualified),counts=q['counts']);persist()
        print(json.dumps(dict(profile=profile,counts=q['counts'],actual_test_images=112)),flush=True)
    state.update(status='independently_qualified_current_default111_both_profiles',active_profile=None,source_and_corresponding_QA_frozen=False)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    state['finished_utc']=datetime.now(timezone.utc).isoformat();persist()
print(json.dumps(dict(status=state['status'],output=str(out)),ensure_ascii=False),flush=True)
