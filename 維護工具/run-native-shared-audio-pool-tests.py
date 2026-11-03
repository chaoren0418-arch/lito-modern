"""Compile candidate modules against retained Debug dependencies, never product source."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import ctypes, hashlib, json, os, re, subprocess, zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
candidate=ROOT/'維護工具/candidates/native-selector-nine-integration-shared-pool'
metadata=read(candidate/'candidate.json')
base=Path(metadata['base_related_run'])
q=read(base/'completion-observation.json');pins=read(base/'compiled-inputs.json')
assert q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_all_terminal']
assert len(pins)==1025 and q['source_manifest_sha256']==metadata['base_source_manifest_sha256']
assert not metadata['applied']
for name,h in metadata['after'].items():assert sha(candidate/'sources'/Path(name).name)==h
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-shared-audio-pool-tests'/stamp
out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-shared-audio-pool-tests'/stamp
runtime.mkdir(parents=True)
probe=candidate/'isolated_tests.rs'
fixtures=candidate/'candidate.json'
for name in ['isolated_tests.rs','candidate.json','before.zip']:
    (out/name).write_bytes((candidate/name).read_bytes())
for name in metadata['after']:
    (out/Path(name).name).write_bytes((candidate/'sources'/Path(name).name).read_bytes())
(out/'audio_output-original.rs').write_bytes((PROJECT/'src/modern_lito/audio_output.rs').read_bytes())
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
candidate_pins={str(p):sha(p) for p in candidate.rglob('*') if p.is_file()}
libraries,native={},set()
for line in (base/'Debug/build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    m=json.loads(line)
    if m.get('reason')=='build-script-executed':native.update(m.get('linked_paths',[]))
    if m.get('reason')=='compiler-artifact' and m['target']['name'] in ['lito_modern','serde_json','serde','anyhow','sha2','windows']:
        paths=[p for p in m['filenames'] if p.endswith('.rlib')]
        if paths:libraries[m['target']['name']]=paths[0]
assert set(libraries)=={'lito_modern','serde_json','serde','anyhow','sha2','windows'}
state=dict(status='preparing_isolated_shared_audio_pool_tests',base_related_run=str(base),
    source_manifest_sha256=q['source_manifest_sha256'],candidate_pins=candidate_pins,
    libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},fixture_sha256=sha(fixtures),
    process_history=[],owned_process=None,product_source_changed=False,applied=False,
    actual_new_sessions=0,actual_new_casts=0,GUI_verified=False,whole_goal_complete=False)
def stable():
    assert all(sha(ROOT/n)==h for n,h in pins.items())
    assert all(sha(p)==h for p,h in candidate_pins.items())
    assert all(sha(v['path'])==v['sha256'] for v in state['libraries'].values())
    assert sha(fixtures)==state['fixture_sha256']
env=dict(os.environ,CARGO_MANIFEST_DIR=str(PROJECT),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',
    TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
def execute(args,folder):
    stable();folder.mkdir()
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        k=ctypes.WinDLL('kernel32',use_last_error=True)
        k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
        k.GetProcessTimes.restype=wintypes.BOOL
        times=[wintypes.FILETIME() for _ in range(4)]
        assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
        ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        item=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=args)
        state['process_history'].append(item);state['owned_process']=item;save(out/'receipt.json',state)
        code=p.wait();item['exit_code']=code;state['owned_process']=None;save(out/'receipt.json',state)
    stable();return code
exe=runtime/'selector-nine-intents.exe'
args=[str(RUST/'rustc.exe'),'--test',str(probe),'--edition=2021','--crate-name','native_shared_audio_pool_tests',
    '--target','x86_64-pc-windows-msvc','-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),
    '-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
for n,p in libraries.items():args+=['--extern',n+'='+p]
for p in sorted(native):args+=['-L',p]
save(out/'receipt.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
error=None
try:
    state['status']='compiling_isolated_shared_audio_pool_tests'
    assert execute(args,out/'build')==0,'Preserve compile failure before repair'
    state.update(executable=str(exe),executable_sha256=sha(exe),status='executing_isolated_shared_audio_pool_tests')
    assert execute([str(exe),'--list'],out/'inventory')==0
    names=re.findall(r'^(.+): test$',(out/'inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M)
    assert len(names)==len(set(names)) and len(names)>=20
    required={"modern_lito::combat_audio::output::tests::legacy_and_native_share_same_id_guard_and_keep_namespace_validation",
        "modern_lito::combat_audio::output::tests::legacy_start_fault_latches_both_legacy_and_native_admission"}
    assert required<=set(names)
    assert execute([str(exe),'--test-threads=1'],out/'runtime')==0
    text=(out/'runtime/stdout.log').read_text(encoding='utf-8-sig')
    counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
    assert len(counts)==1
    passed,failed,ignored=map(int,counts[0]);assert failed==0 and passed+ignored==len(names)
    ok=set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))
    skipped=set(re.findall(r'^test (.+?) \.\.\. ignored(?:,.*)?$',text,re.M))
    assert required<=ok and ok|skipped==set(names) and len(ok)==passed and len(skipped)==ignored
    diagnostics=(out/'runtime/stderr.log').read_text(encoding='utf-8-sig').splitlines()
    assert diagnostics==[
        'LiTo audio shutdown: waveOutReset failed (MMRESULT 5); retaining driver-owned buffer for safety',
        'LiTo audio shutdown: waveOutUnprepareHeader failed (MMRESULT 5); retaining driver-owned buffer for safety',
        'LiTo audio shutdown: waveOutClose failed (MMRESULT 5); retaining driver-owned buffer for safety',
    ], 'Only the three exact forced Fake-driver shutdown diagnostics are admitted'
    state.update(status='passed_isolated_shared_audio_pool_tests',passed=passed,failed=failed,ignored=ignored)
except BaseException as e:
    error=e;state.update(status='failed_preserved',error=repr(e))
finally:save(out/'receipt.json',state)
files={p.relative_to(out).as_posix():p for p in out.rglob('*') if p.is_file()}
files['source-metadata.json']=fixtures
if exe.exists():files['candidate.exe']=exe
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(hashes)
    for n,h in hashes.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(status=state['status'],source_manifest_sha256=q['source_manifest_sha256'],
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,product_source_changed=False,applied=False,
    actual_new_sessions=0,actual_new_casts=0,GUI_verified=False,whole_goal_complete=False))
print(json.dumps(dict(output=str(out),status=state['status'],raw_archive_sha256=sha(out/'raw.zip')),ensure_ascii=False))
if error:raise error
