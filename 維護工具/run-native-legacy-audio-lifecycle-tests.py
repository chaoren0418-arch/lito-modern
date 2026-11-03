"""Run two real-resource lifecycle regressions against the pinned candidate libraries."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import ctypes, hashlib, json, os, re, subprocess, zipfile
ROOT=Path(__file__).resolve().parents[1]; P=ROOT/'LiTo381-Modern'; V=ROOT/'驗證紀錄/current'
def read(p): return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v): Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
family=V/'native1040-teleport-legacy-audio'
compiles={'Debug':family/'compile-20261018-102913-190473-debug-01c39fac',
          'Release':family/'compile-20261018-101848-311229-release-b5342797'}
qualifications={'Debug':family/'run-20261018-103201-021942/qualified-20261018-103433-736263/qualified.json',
                'Release':family/'run-20261018-102231-473964/qualified-20261018-102646-838181/qualified.json'}
inputs=read(compiles['Debug']/'compiled-inputs.json'); assert len(inputs)==1041
candidate=ROOT/'維護工具/candidates/native-legacy-audio-lifecycle-tests'
prepared=read(candidate/'preparation.json')
assert sha(prepared['original_source'])==prepared['original_sha256']
# rustfmt is part of preparation; pin its final bytes, rather than its input.
prepared['candidate_sha256']=sha(candidate/'tests.rs'); save(candidate/'preparation.json',prepared)
def guard():
    q=read(ROOT/'工作佇列.json'); assert not q.get('active_workloads') and not q.get('source_and_corresponding_QA_frozen')
    for name,h in inputs.items(): assert sha(ROOT/name)==h,name
    assert sha(candidate/'tests.rs')==prepared['candidate_sha256']
    for profile,folder in compiles.items():
        b=read(folder/'receipt.json'); assert b['status']=='compiled_candidate_legacy_audio_lifecycle_library_and_probe'
        assert b['candidate_effective_manifest_sha256']=='8A638CE7EE862A933A7B9E610783FCAB32B15A69D5A9482A9CE441AD1CFAE55E'
        assert sha(b['diagnostic_library'])==b['diagnostic_library_sha256']
        assert read(folder/'compile-terminal-observation.json')['observed_terminal_exit_code']==0
        assert qualifications[profile].is_file()
        for pin in b['candidate_inputs'].values(): assert sha(pin['path'])==pin['sha256']
        for pin in b['libraries'].values(): assert sha(pin['path'])==pin['sha256']
guard()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=V/'native-legacy-audio-lifecycle-tests'/stamp; out.mkdir(parents=True)
runtime=P/'target/current/native-legacy-audio-lifecycle-tests'/stamp; runtime.mkdir(parents=True)
for name in ['tests.rs','preparation.json']: (out/name).write_bytes((candidate/name).read_bytes())
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True); k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    t=[wintypes.FILETIME() for _ in range(4)]; assert k.GetProcessTimes(handle,*[ctypes.byref(v) for v in t])
    ticks=t[0].dwLowDateTime|(t[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='running_candidate_real_resource_lifecycle_regressions',runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),
           candidate_test_sha256=sha(out/'tests.rs'),process_history=[],profiles={},normal_product_source_modified=False,
           diagnostic_API_extension_present=True,normal_GUI_API_permission_inferred=False,new_sessions=0,physical_audio_verified=False,whole_goal_complete=False)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
def execute(args,label):
    with (out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
        child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=args)
        state['process_history'].append(job); state['owned_process']=job; save(out/'receipt.json',state)
        job['exit_code']=child.wait(); state['owned_process']=None; save(out/'receipt.json',state)
        assert job['exit_code']==0,label
print('LEGACY_LIFECYCLE_TEST_RUN='+str(out),flush=True)
try:
    for profile,folder in compiles.items():
        guard(); b=read(folder/'receipt.json'); args=b['process_history'][-1]['arguments']
        common=args[args.index('--edition=2021'):]; exe=runtime/(profile.lower()+'-resource-lifecycle.exe')
        command=[args[0],str(out/'tests.rs'),'--test','--crate-name','native_legacy_audio_lifecycle_tests',
                 '--extern','lito_modern='+b['diagnostic_library'],'-o',str(exe)]+common
        execute(command,profile+'-compile'); execute([str(exe),'--list'],profile+'-inventory')
        names=re.findall(r'^(.+): test$',(out/(profile+'-inventory.stdout.txt')).read_text('utf-8-sig'),re.M)
        selected=[name for name in names if 'legacy_' in name]; assert len(selected)==2,selected
        execute([str(exe),'legacy_','--test-threads=1'],profile+'-runtime')
        text=(out/(profile+'-runtime.stdout.txt')).read_text('utf-8-sig')
        expected=f'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; {len(names)-2} filtered out;'
        assert expected in text,text
        assert set(re.findall(r'^test (.+) \.\.\. ok$',text,re.M))==set(selected)
        state['profiles'][profile]=dict(compile=str(folder),compile_receipt_sha256=sha(folder/'receipt.json'),
            previous_qualification=str(qualifications[profile]),previous_qualification_sha256=sha(qualifications[profile]),
            diagnostic_library=b['diagnostic_library'],diagnostic_library_sha256=b['diagnostic_library_sha256'],
            product=str(exe),product_sha256=sha(exe),passed=2,failed=0,ignored=0,filtered_out=len(names)-2,test_names=selected)
        save(out/'receipt.json',state)
    guard(); state['status']='executed_both_profiles_candidate_real_resource_lifecycle_regressions'
except BaseException as error: state.update(status='failed_preserved',error=repr(error)); raise
finally: save(out/'receipt.json',state)
raw={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as archive:
    for name in raw: archive.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==h for name,h in raw.items())
save(out/'readback.json',dict(files=raw,archive_sha256=sha(out/'evidence.zip')))
print(json.dumps(dict(run=str(out),profiles={p:dict(passed=2,failed=0,ignored=0) for p in state['profiles']},new_sessions=0),ensure_ascii=False))
