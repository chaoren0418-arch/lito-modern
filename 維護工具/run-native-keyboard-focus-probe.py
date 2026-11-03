"""Run a small owned-control API regression without changing the normal product."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import ctypes, hashlib, json, os, subprocess, zipfile
R = Path(__file__).resolve().parents[1]; P = R / 'LiTo381-Modern'; V = R / '驗證紀錄/current'
C = R / '維護工具/candidates/native-keyboard-focus-probe'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p): return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def save(p, d): Path(p).write_text(json.dumps(d, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')
k = ctypes.WinDLL('kernel32', use_last_error=True); k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)]; assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts])
    ticks=ts[0].dwLowDateTime|ts[0].dwHighDateTime<<32
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
queue=read(R/'工作佇列.json'); assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
assert queue['current_full_default_library_verified']
library=Path(queue['current_native_catalog_library']['run']); inputs=read(library/'compiled-inputs.json')
assert len(inputs)==1048
for name,h in inputs.items(): assert sha(R/name)==h,name
out=V/'native-keyboard-focus-probe'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
pins={name:sha(C/name) for name in ['Cargo.toml','Cargo.lock','src/main.rs']}
with zipfile.ZipFile(out/'candidate-source.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in pins:z.write(C/name,name)
with zipfile.ZipFile(out/'candidate-source.zip') as z:
    assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in pins.items())
original=R/'LiTo381-Modern/src/modern_lito/native_login_window.rs'
(out/'normal-native-login-window.before.rs').write_bytes(original.read_bytes())
state=dict(status='prepared_isolated_API_probe', output=str(out), normal_source_manifest_sha256=queue['current_product_identity']['source_manifest_sha256'],
    normal_UI_source_sha256=sha(original), normal_library=str(library), normal_product_modified=False,
    normal_GUI_qualified=False, candidate_pins=pins, candidate_archive_sha256=sha(out/'candidate-source.zip'),
    pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())), processes=[], whole_goal_complete=False)
persist=lambda:save(out/'receipt.json',state)
env=os.environ.copy();env.update(RUSTC=str(RUST/'rustc.exe'), RUSTDOC=str(RUST/'rustdoc.exe'),
    TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
env['PATH']=str(RUST)+os.pathsep+env['PATH']
assert not {key:value for key,value in env.items() if value and (key.startswith('CARGO_PROFILE_') or key in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
def execute(command, label, timeout):
    with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:
        child=subprocess.Popen(command,cwd=C,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        row=dict(label=label,pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command)
        state['processes'].append(row);persist()
        try: code=child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            child.kill();child.wait();row.update(timed_out=True,exit_code=child.returncode);persist();raise
        row['exit_code']=code;persist();return code
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_isolated_API_probe';persist()
    command=[str(RUST/'cargo.exe'),'build','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc',
             '--target-dir',str(P/'target/current'),'--message-format=json']
    assert execute(command,'build',180)==0,'Isolated probe compiler failure retained'
    messages=[json.loads(line) for line in (out/'build.stdout.log').read_text('utf-8-sig').splitlines() if line.startswith('{')]
    artifacts=[m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable')]
    assert len(artifacts)==1
    exe=Path(artifacts[0]['executable']).resolve(strict=True);assert exe.is_relative_to(P/'target/current')
    state.update(status='running_isolated_API_probe',actual_exe=str(exe),actual_exe_sha256=sha(exe));persist()
    assert execute([str(exe)],'runtime',20)==0,'Isolated API hypothesis not confirmed; do not apply product changes'
    rows=[json.loads(line) for line in (out/'runtime.stdout.log').read_text('utf-8-sig').splitlines() if line.startswith('{')]
    assert len(rows)==2 and rows[0]['original_EnableWindow_focus_became_null']
    assert rows[1]['status']=='isolated_owned_control_API_regression_reproduced_and_candidate_checked'
    assert all(rows[1][name] for name in ['background_preserved','closing_preserved','parent_restored','enabled_control_preserved'])
    state.update(status='isolated_owned_control_focus_loss_reproduced_candidate_API_checks_passed', result=rows)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    for name,h in inputs.items(): assert sha(R/name)==h,name
    assert all(sha(C/name)==h for name,h in pins.items())
    state['normal_source_inputs_unchanged']=True;state['finished_utc']=datetime.now(timezone.utc).isoformat();persist()
print(json.dumps(dict(output=str(out),status=state['status'],result=state['result'],normal_product_modified=False),ensure_ascii=False))
