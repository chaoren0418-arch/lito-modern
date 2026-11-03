"""Compile only the proposed Rust increment port and replay qualified native vectors."""
from pathlib import Path
from datetime import datetime,timezone
import csv,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
ORACLE=ROOT/'驗證紀錄/current/native-background-camera-selector/20261016-231342-773145'
CURRENT=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
RUSTC=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rustc.exe')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
qualified=ORACLE/'qualified-origin-corrected.json';q=read(qualified)
assert q['status']=='independently_qualified_original_camera_increment_selector_fragment' and q['cases']==192
assert q['original_session_id'] is None and q['original_exec_tool_chunk_id']=='6cefd6'
assert sha(ORACLE/'vectors.csv')==q['raw_files']['vectors.csv']
pins=read(CURRENT/'compiled-inputs.json');assert len(pins)==1025
source=ROOT/'維護工具/candidates/native-background-camera/native_background_camera.rs'
probe=ROOT/'維護工具/oracle/NativeBackgroundCameraRustProbe.rs'
source_hash=sha(source);probe_hash=sha(probe)
def stable():
    for name,h in pins.items():assert sha(ROOT/name)==h
    assert sha(source)==source_hash and sha(probe)==probe_hash
stable()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-background-camera-rust-candidate'/stamp
runtime=PROJECT/'target/current/native-background-camera-rust-candidate'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
for name,path in [('runner.py',Path(__file__)),('candidate.rs',source),('probe.rs',probe),('native-vectors.csv',ORACLE/'vectors.csv'),('native-qualification.json',qualified)]:
    (out/name).write_bytes(path.read_bytes())
(out/'compiled-inputs.json').write_bytes((CURRENT/'compiled-inputs.json').read_bytes())
exe=runtime/'rust-camera-candidate.exe'
command=[str(RUSTC),str(out/'probe.rs'),'--edition=2021','--crate-name','native_background_camera_rust_candidate','--target','x86_64-pc-windows-msvc','-o',str(exe)]
state=dict(status='prepared_isolated_Rust_camera_increment_candidate',output=str(out),runtime=str(runtime),
    native_qualification=str(qualified),native_qualification_sha256=sha(qualified),native_vectors_sha256=sha(ORACLE/'vectors.csv'),
    source_manifest_sha256=q['source_manifest_sha256'],candidate_path=str(source),candidate_sha256=source_hash,probe_sha256=probe_hash,
    compiler_path=str(RUSTC),compiler_sha256=sha(RUSTC),compiler_arguments=command,
    product_source_changed=False,modern_Pose_binding_verified=False,whole_callback_executed=False,
    actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False,processes=[],owned_process=None)
persist=lambda:save(out/'receipt.json',state)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb') as stdout,(out/(label+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['processes'].append(item);state['owned_process']=item;persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(label,code,'Preserve original failure before repair')
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_isolated_Rust_camera_increment_candidate';persist();execute(command,'compile')
    binary=exe.read_bytes();pe=int.from_bytes(binary[60:64],'little')
    assert binary[:2]==b'MZ' and binary[pe+4:pe+6]==b'\x64\x86' and binary[pe+24:pe+26]==b'\x0b\x02'
    state['status']='replaying_qualified_original_camera_vectors_against_Rust';persist()
    execute([str(exe),str(out/'native-vectors.csv'),str(out/'candidate-results.json')],'native-vectors')
    result=read(out/'candidate-results.json');assert result['cases']==192 and result['checked_host_domain_rejections']==10
    rows=list(csv.DictReader((out/'native-vectors.csv').open(encoding='ascii',newline='')))
    assert len(rows)==192 and len({(v['count'],v['heading'],v['actor13'],v['before_x'],v['before_y']) for v in rows})==192
    stable();state.update(status='isolated_Rust_camera_increment_candidate_passed',cases=192,checked_host_domain_rejections=10,
        diagnostic_sha256=sha(exe),report_sha256=sha(out/'candidate-results.json'),finished_utc=datetime.now(timezone.utc).isoformat());persist()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if exe.exists():files['Rust-x64-candidate-probe.exe']=exe
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,h in hashes.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status'],cases=state.get('cases')),ensure_ascii=False),flush=True)
