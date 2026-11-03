"""Continue after the original Debug batch; never repeat its tests."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,sys
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def born(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
ap=argparse.ArgumentParser();ap.add_argument('--debug-run',type=Path,required=True);ap.add_argument('--debug-session',type=int,required=True);ap.add_argument('--debug-terminal-chunk',required=True);a=ap.parse_args()
debug=a.debug_run.resolve(strict=True);assert debug.is_relative_to(ROOT/'驗證紀錄/current/default-native-regression')
d=read(debug/'receipt.json');dq=debug/'profile-qualified/qualified.json';q=read(dq)
assert d['profile']==q['profile']=='Debug' and d['status']=='verified_current_default111_integration_targets_and_main_units'
assert q['status']=='independently_qualified_current_default111_integrations_and_main' and q['counts']['failed']==0 and q['actual_test_executables']==112
assert a.debug_terminal_chunk not in {'pending','unknown',''}
source=q['source_manifest_sha256'];assert source=='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182'
normal=Path(d['normal_qualification']);startup=normal.parent.parent;inputs=read(startup/'compiled-inputs.json')
assert len(inputs)==1031 and len(read(Path(read(normal)['identity_manifest'])))==1030
g06=Path(d['G06_qualification']);g=read(g06);assert g['actual_formal_sessions']==96 and g['source_manifest_sha256']==source and g['observed_terminal_exit_code']==0
names=['run-native1030-default111.py','qualify-native1030-default111.py','preserve-legacy-library-reports.py',Path(__file__).name]
qa={n:sha(ROOT/'維護工具'/n) for n in names}
out=ROOT/'驗證紀錄/current/native1030-default111'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n in names:(out/n).write_bytes((ROOT/'維護工具'/n).read_bytes())
state=dict(status='prepared_Release_after_qualified_original_Debug',output=str(out),pipeline_pid=os.getpid(),pipeline_creation_utc=born(k.GetCurrentProcess()),source_manifest_sha256=source,source_files=1030,compiled_input_files=1031,normal_qualification=str(normal),normal_qualification_sha256=sha(normal),G06_qualification=str(g06),G06_qualification_sha256=sha(g06),profiles={'Debug':dict(run=str(debug),actual_runner_exit_code=0,original_session_id=a.debug_session,terminal_tool_chunk=a.debug_terminal_chunk,qualified=True,qualification=str(dq),qualification_sha256=sha(dq),counts=q['counts'])},validation_pins=qa,owned_process=None,process_history=[],source_and_corresponding_QA_frozen=True,full_library_in_this_scope=False,GUI_verified=False,whole_goal_complete=False)
persist=lambda:save(out/'cohort.json',state)
def check():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in qa.items():assert sha(ROOT/'維護工具'/n)==h,n
    assert sha(dq)==state['profiles']['Debug']['qualification_sha256']
    for profile in ['Debug','Release']:
        original=read(next(startup.glob(profile+'-qualified-*/qualified.json')))
        assert original['source_manifest_sha256']==source and sha(original['normal']['path'])==original['normal']['sha256']
env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
def execute(args,folder):
    folder.mkdir();check()
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(args,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        record=dict(pid=p.pid,creation_utc=born(wintypes.HANDLE(int(p._handle))),arguments=args,output=str(folder),exit_code=None)
        state['process_history'].append(record);state['owned_process']=record;persist()
        code=p.wait();record['exit_code']=code;record['finished_utc']=datetime.now(timezone.utc).isoformat();state['owned_process']=None;persist()
    check();return code
persist();print('CURRENT1030_DEFAULT111_COHORT='+str(out),flush=True)
try:
    release_normal=next(startup.glob('Release-qualified-*/qualified.json'))
    state.update(status='executing_current_default111_profile',active_profile='Release');persist()
    parent=ROOT/'驗證紀錄/current/default-native-regression';before=set(parent.iterdir())
    code=execute([sys.executable,'-B',str(ROOT/'維護工具/run-native1030-default111.py'),'--profile','Release','--normal-qualification',str(release_normal),'--G06-qualification',str(g06)],out/'Release-console')
    created=set(parent.iterdir())-before;assert len(created)==1;run=created.pop()
    state['profiles']['Release']=dict(run=str(run),actual_runner_exit_code=code,qualified=False);persist()
    assert code==0,'Preserve original Release failure; do not restart.'
    state['status']='independently_reading_current_default111_profile';persist()
    assert execute([sys.executable,'-B',str(ROOT/'維護工具/qualify-native1030-default111.py'),str(run)],out/'Release-readback')==0
    rq=run/'profile-qualified/qualified.json';r=read(rq);assert r['source_manifest_sha256']==source and r['actual_test_executables']==112 and r['counts']['failed']==0
    state['profiles']['Release'].update(qualified=True,qualification=str(rq),qualification_sha256=sha(rq),counts=r['counts']);persist()
    state.update(status='independently_qualified_current_default111_both_profiles',active_profile=None,source_and_corresponding_QA_frozen=False)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();persist()
print(json.dumps(dict(status=state['status'],output=str(out),counts={p:j['counts'] for p,j in state['profiles'].items()}),ensure_ascii=False),flush=True)
