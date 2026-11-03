"""Compile original Release G06 targets as a control for the retained Debug failure."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
FAILED=ROOT/'驗證紀錄/current/native-G06-controller/20261017-030505-804788'
PAIR=ROOT/'驗證紀錄/current/native-detection-invisibility/qualified-pair-20261017-030425-432673'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
prior=read(FAILED/'failure-before-repair/readback.json');assert prior['original_session_id']==45538 and prior['observed_terminal_exit_code']==1
assert sha(FAILED/'failure-before-repair/raw.zip')==prior['raw_archive_sha256']
pins=read(PAIR/'compiled-inputs.json');assert len(pins)==1025
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
stable();out=FAILED/('Release-control-build-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'compiled-inputs.json').write_bytes((PAIR/'compiled-inputs.json').read_bytes())
command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc',
    '--target-dir','target/current','--message-format=json','--release','--test','native_teleport_handoff','--test','native_teleport_failure']
state=dict(status='compiling_original_Release_G06_control_targets',source_manifest_sha256=prior['source_manifest_sha256'],
    retained_failed_run=str(FAILED),retained_failed_archive_sha256=prior['raw_archive_sha256'],output=str(out),
    compiler_arguments=command,profile='Release',product_source_changed=False,actual_new_sessions=0,G06_qualified=False,
    performance_accepted=False,whole_goal_complete=False,owned_process=None)
persist=lambda:save(out/'build.json',state)
env=dict(os.environ,PATH=str(RUST)+os.pathsep+os.environ['PATH'],RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),
    TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
persist();print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    p=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    times=[wintypes.FILETIME() for _ in range(4)];k=ctypes.WinDLL('kernel32',use_last_error=True)
    k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
    assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
    ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
    job=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=command)
    state['owned_process']=job;persist();code=p.wait();job['exit_code']=code;state['process']=job;state['owned_process']=None
state.update(status='compiled_original_Release_G06_control_targets' if code==0 else 'control_compile_failure_preserved',compile_exit_code=code)
persist();stable();assert code==0,'Preserve actual compile failure before any repair'
messages=[json.loads(l) for l in (out/'stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
products={}
for m in messages:
    if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']:
        assert m['features']==[] and m['profile']['opt_level']=='z' and not m['profile']['debug_assertions']
        products[m['target']['name']]=dict(path=m['executable'],sha256=sha(m['executable']),compiler_profile=m['profile'])
assert set(products)=={'native_teleport_handoff','native_teleport_failure'}
state.update(products=products,source_inputs_unchanged=True,finished_utc=datetime.now(timezone.utc).isoformat());persist()
print(json.dumps(dict(output=str(out),compile_exit_code=code,products=products),ensure_ascii=False),flush=True)
