"""Prove the old-root Model reuse bug before applying the Presentation change."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--related-run',required=True,type=Path);a=ap.parse_args()
related=a.related_run.resolve(strict=True);assert related.is_relative_to(ROOT/'驗證紀錄/current/native-shared-archive-owner')
q=read(related/'completion-observation.json');assert q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_all_terminal']
pins=read(related/'compiled-inputs.json');assert len(pins)==1025
candidate=ROOT/'維護工具/candidates/native-presentation-source-owner';meta=read(candidate/'candidate.json')
for n,h in meta['before'].items():assert sha(ROOT/n)==h
source=candidate/'native_world_lighting_scene.rs';assert sha(source)==meta['after']['LiTo381-Modern/tests/native_world_lighting_scene.rs']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-presentation-source-owner'/('red-'+stamp);out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-presentation-source-owner-red'/stamp;runtime.mkdir(parents=True)
text=source.read_text(encoding='utf-8');old='include_bytes!("fixtures/native-world-current.json")'
assert text.count(old)==1;text=text.replace(old,'include_bytes!(r"'+str(PROJECT/'tests/fixtures/native-world-current.json')+'")')
probe=out/'red-test.rs';probe.write_text(text,encoding='utf-8',newline='\n');assert bytes([0]) not in probe.read_bytes()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'candidate.json').write_bytes((candidate/'candidate.json').read_bytes())
state=dict(status='preparing_root_reuse_red_test',output=str(out),source_manifest_sha256=q['source_manifest_sha256'],
    prerequisite=str(related/'completion-observation.json'),prerequisite_sha256=sha(related/'completion-observation.json'),
    test_source_sha256=sha(probe),registered_candidate_test_source_sha256=sha(source),
    product_sources_changed=False,process_history=[],owned_process=None,GUI_verified=False,whole_goal_complete=False)
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    assert sha(source)==state['registered_candidate_test_source_sha256'] and sha(probe)==state['test_source_sha256']
def execute(args,folder):
    stable();folder.mkdir()
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];k=ctypes.WinDLL('kernel32',use_last_error=True)
        k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
        assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
        ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        item=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=args)
        state['process_history'].append(item);state['owned_process']=item;save(out/'receipt.json',state)
        code=p.wait();item['exit_code']=code;state['owned_process']=None;save(out/'receipt.json',state)
    stable();return code
env=dict(os.environ,CARGO_MANIFEST_DIR=str(PROJECT),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
libraries={};native=set()
for line in (related/'Debug/build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    m=json.loads(line)
    if m.get('reason')=='build-script-executed':native.update(m.get('linked_paths',[]))
    if m.get('reason')=='compiler-artifact' and m['target']['name'] in ['lito_modern','serde_json']:
        files=[p for p in m['filenames'] if p.endswith('.rlib')]
        if files:libraries[m['target']['name']]=files[0]
assert set(libraries)=={'lito_modern','serde_json'}
state['libraries']={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()}
exe=runtime/'presentation-root-red.exe'
args=[str(RUST/'rustc.exe'),'--test',str(probe),'--edition=2021','--crate-name','native_presentation_root_red','--target','x86_64-pc-windows-msvc',
    '-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
for n,p in libraries.items():args+=['--extern',n+'='+p]
for p in sorted(native):args+=['-L',p]
stable();save(out/'receipt.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_root_reuse_red_test';assert execute(args,out/'build')==0,'Retain red preparation failure before correction'
    state.update(executable=str(exe),executable_sha256=sha(exe),status='executing_root_reuse_red_test')
    code=execute([str(exe),'presentation_job_rejects_reused_model_from_other_resource_root','--exact','--nocapture','--test-threads=1'],out/'run')
    state['test_exit_code']=code;assert code==101,'Expected the unchanged baseline to reject the new wrong-root assertion'
    assert 'Job reused an old-root Model' in (out/'run/stderr.log').read_text(encoding='utf-8-sig')
    state['status']='expected_old_root_reuse_failure_observed'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
files={p.relative_to(out).as_posix():p for p in out.rglob('*') if p.is_file()};files['red-test.exe']=exe
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'red-raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'red-raw.zip') as z:
    assert set(z.namelist())==set(hashes)
    for n,h in hashes.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'raw-readback.json',dict(status=state['status'],source_manifest_sha256=q['source_manifest_sha256'],test_exit_code=101,
    raw_archive_sha256=sha(out/'red-raw.zip'),raw_files=hashes,product_sources_changed=False,whole_goal_complete=False))
print(json.dumps(dict(status=state['status'],test_exit_code=101,raw_sha256=sha(out/'red-raw.zip')),ensure_ascii=False))
