"""Replay native offset vectors and synthetic queue scenarios against an unapplied Core."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
BUILD=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
ORACLE=ROOT/'驗證紀錄/current/native-background-offset-callback/20261016-233627-359668'
CANDIDATE=ROOT/'維護工具/candidates/native-background-clock'
RUSTC=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rustc.exe')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
q=read(ORACLE/'qualified.json');assert q['status']=='independently_qualified_original_background_offset_callback' and q['cases']==57
assert sha(ORACLE/'vectors.csv')==q['raw_files']['vectors.csv']
pins=read(BUILD/'compiled-inputs.json');assert len(pins)==1025
meta=read(CANDIDATE/'metadata.json');assert sha(meta['base_source'])==sha(CANDIDATE/'base-native_water_clock.rs')==meta['base_sha256']
assert sha(CANDIDATE/'native_water_clock.rs')==meta['candidate_sha256']
deps=PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps';libraries={}
for line in (BUILD/'Debug/build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    m=json.loads(line)
    if m.get('reason')=='compiler-artifact' and m['target']['name'] in ['serde','serde_json','anyhow']:
        paths=[Path(s) for s in m['filenames'] if s.endswith('.rlib') and Path(s).parent==deps]
        if paths:assert len(paths)==1;libraries[m['target']['name']]=paths[0]
assert set(libraries)=={'serde','serde_json','anyhow'}
candidate_files={n:sha(CANDIDATE/n) for n in ['base-native_water_clock.rs','native_water_clock.rs','metadata.json']}
probe=ROOT/'維護工具/oracle/NativeBackgroundClockRustProbe.rs';probe_sha=sha(probe)
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h
    for n,h in candidate_files.items():assert sha(CANDIDATE/n)==h
    assert sha(probe)==probe_sha
stable();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-background-clock-rust-candidate'/stamp
runtime=PROJECT/'target/current/native-background-clock-rust-candidate'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
for n,p in [('runner.py',Path(__file__)),('probe.rs',probe),('native-vectors.csv',ORACLE/'vectors.csv'),('native-qualification.json',ORACLE/'qualified.json'),('compiled-inputs.json',BUILD/'compiled-inputs.json')]:
    (out/n).write_bytes(p.read_bytes())
for n in candidate_files:(out/n).write_bytes((CANDIDATE/n).read_bytes())
exe=runtime/'background-clock-candidate.exe'
command=[str(RUSTC),str(out/'probe.rs'),'--edition=2021','--crate-name','native_background_clock_candidate','--target','x86_64-pc-windows-msvc','-L','dependency='+str(deps),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
for n,p in libraries.items():command+=['--extern',n+'='+str(p)]
state=dict(status='prepared_unapplied_background_shared_queue_candidate',output=str(out),runtime=str(runtime),
    source_manifest_sha256=q['source_manifest_sha256'],native_qualification=str(ORACLE/'qualified.json'),native_qualification_sha256=sha(ORACLE/'qualified.json'),
    native_vectors_sha256=sha(ORACLE/'vectors.csv'),candidate_files=candidate_files,probe_sha256=probe_sha,
    compiler_arguments=command,libraries={n:dict(path=str(p),sha256=sha(p)) for n,p in libraries.items()},
    candidate_applied_to_product=False,product_source_changed=False,actual_scene_start_stop_binding_verified=False,
    real_clock_or_GUI_timing_verified=False,actual_formal_sessions=0,whole_goal_complete=False,processes=[],owned_process=None)
persist=lambda:save(out/'receipt.json',state)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb') as stdout,(out/(label+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['processes'].append(item);state['owned_process']=item;persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(label,code,'Preserve failure before repair')
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_unapplied_shared_background_clock_candidate';persist();execute(command,'compile')
    binary=exe.read_bytes();h=int.from_bytes(binary[60:64],'little')
    assert binary[:2]==b'MZ' and binary[h+4:h+6]==b'\x64\x86' and binary[h+24:h+26]==b'\x0b\x02'
    state['status']='replaying_native_callback_and_synthetic_queue_vectors';persist()
    execute([str(exe),str(out/'native-vectors.csv'),str(out/'candidate-results.json')],'vectors')
    result=read(out/'candidate-results.json');assert result['native_callback_cases']==57 and len(result['synthetic_queue_scenarios'])==7
    stable()
    for info in state['libraries'].values():assert sha(info['path'])==info['sha256']
    state.update(status='unapplied_background_shared_clock_candidate_passed',native_callback_cases=57,synthetic_queue_scenarios=7,
        diagnostic_sha256=sha(exe),report_sha256=sha(out/'candidate-results.json'));persist()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if exe.exists():files['Rust-x64-probe.exe']=exe
    for name,p in libraries.items():files['libraries/'+p.name]=p
    hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for n,h in hashes.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status'],native_callback_cases=state.get('native_callback_cases'),synthetic_queue_scenarios=state.get('synthetic_queue_scenarios')),ensure_ascii=False),flush=True)
