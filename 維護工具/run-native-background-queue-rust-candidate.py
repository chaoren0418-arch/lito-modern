"""Compare the unapplied Core with actual original queue/lifecycle snapshots."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
BUILD=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
NATIVE=ROOT/'驗證紀錄/current/native-background-queue-lifecycle/20261016-235558-661668'
CANDIDATE=ROOT/'維護工具/candidates/native-background-clock'
PROBE=ROOT/'維護工具/oracle/NativeBackgroundQueueRustProbe.rs'
RUSTC=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rustc.exe')
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--native-run',type=Path,default=NATIVE);a=ap.parse_args()
NATIVE=a.native_run.resolve(strict=True);assert NATIVE.is_relative_to(ROOT/'驗證紀錄/current/native-background-queue-lifecycle')
q=read(NATIVE/'qualified.json')
assert q['status']=='independently_qualified_original_background_queue_and_selected_lifecycle_slices'
ready=q.get('distinct_ready_reuse_filter_executed',False)
expected_snapshots,expected_rows=(18,90) if ready else (16,74)
assert (q['snapshots'],q['queue_rows'])==(expected_snapshots,expected_rows)
for name in ['state-vectors.csv','queue-vectors.csv']:assert sha(NATIVE/name)==q['raw_files'][name]
prior=ROOT/'驗證紀錄/current/native-background-clock-rust-candidate/20261016-234332-623096/independent-readback.json'
pc=read(prior);assert pc['status']=='independently_read_back_unapplied_shared_background_clock_candidate'
candidate_files=dict(pc['candidate_files']);candidate_files['ready_filter.rs']=sha(CANDIDATE/'ready_filter.rs')
pins=read(BUILD/'compiled-inputs.json');assert len(pins)==1025
deps=PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps';libraries={}
for line in (BUILD/'Debug/build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    m=json.loads(line)
    if m.get('reason')=='compiler-artifact' and m['target']['name'] in ['serde','serde_json','anyhow']:
        paths=[Path(s) for s in m['filenames'] if s.endswith('.rlib') and Path(s).parent==deps]
        if paths:assert len(paths)==1;libraries[m['target']['name']]=paths[0]
assert set(libraries)=={'serde','serde_json','anyhow'}
probe_sha=sha(PROBE)
def stable():
    for name,value in pins.items():assert sha(ROOT/name)==value
    for name,value in candidate_files.items():assert sha(CANDIDATE/name)==value
    assert sha(PROBE)==probe_sha
stable();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-background-queue-rust-candidate'/stamp
runtime=PROJECT/'target/current/native-background-queue-rust-candidate'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
for name,path in [('runner.py',Path(__file__)),('probe.rs',PROBE),('native-state-vectors.csv',NATIVE/'state-vectors.csv'),
                  ('native-queue-vectors.csv',NATIVE/'queue-vectors.csv'),('native-qualification.json',NATIVE/'qualified.json'),
                  ('prior-candidate-qualification.json',prior),('compiled-inputs.json',BUILD/'compiled-inputs.json')]:
    (out/name).write_bytes(path.read_bytes())
for name in candidate_files:(out/name).write_bytes((CANDIDATE/name).read_bytes())
exe=runtime/'background-queue-candidate.exe'
command=[str(RUSTC),str(out/'probe.rs'),'--edition=2021','--crate-name','native_background_queue_candidate','--target','x86_64-pc-windows-msvc',
    '-L','dependency='+str(deps),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
for name,path in libraries.items():command+=['--extern',name+'='+str(path)]
state=dict(status='prepared_unapplied_background_queue_candidate',output=str(out),runtime=str(runtime),
    source_manifest_sha256=q['source_manifest_sha256'],native_qualification=str(NATIVE/'qualified.json'),native_qualification_sha256=sha(NATIVE/'qualified.json'),
    prior_candidate_qualification=str(prior),prior_candidate_qualification_sha256=sha(prior),candidate_files=candidate_files,
    probe_sha256=probe_sha,compiler_arguments=command,libraries={n:dict(path=str(p),sha256=sha(p)) for n,p in libraries.items()},
    candidate_applied_to_product=False,product_source_changed=False,modern_scene_binding_verified=False,
    distinct_ready_reuse_filter_executed=ready,
    clock_source_stubbed=True,actual_formal_sessions=0,whole_goal_complete=False,processes=[],owned_process=None)
def persist():save(out/'receipt.json',state)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb') as stdout,(out/(label+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['processes'].append(item);state['owned_process']=item;persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(label,code,'Preserve failure before repair')
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_unapplied_background_queue_candidate';persist();execute(command,'compile')
    binary=exe.read_bytes();h=int.from_bytes(binary[60:64],'little')
    assert binary[:2]==b'MZ' and binary[h+4:h+6]==b'\x64\x86' and binary[h+24:h+26]==b'\x0b\x02'
    state['status']='comparing_with_native_queue_lifecycle_snapshots';persist();execute([str(exe),str(NATIVE),str(out)],'vectors')
    result=read(out/'candidate-results.json');assert (result['snapshots'],result['queue_rows'])==(expected_snapshots,expected_rows)
    for prefix in ['state','queue']:
        assert (out/('Rust-'+prefix+'-vectors.csv')).read_text().replace('\r\n','\n')==(NATIVE/(prefix+'-vectors.csv')).read_text().replace('\r\n','\n')
    stable()
    for info in state['libraries'].values():assert sha(info['path'])==info['sha256']
    state.update(status='unapplied_background_queue_candidate_matches_native_selected_lifecycle',snapshots=expected_snapshots,queue_rows=expected_rows,
        diagnostic_sha256=sha(exe),report_sha256=sha(out/'candidate-results.json'));persist()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if exe.exists():files['Rust-x64-probe.exe']=exe
    for name,p in libraries.items():files['libraries/'+p.name]=p
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,value in hashes.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
    save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status'],snapshots=state.get('snapshots'),queue_rows=state.get('queue_rows')),ensure_ascii=False),flush=True)
