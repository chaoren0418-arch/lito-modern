"""Current source1040 full library selections; preserve historical outputs."""
from pathlib import Path
import time
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
now = lambda: datetime.now(timezone.utc).isoformat()

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, v):
    p.write_text(json.dumps(v, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('default_cohort', type=Path)
ap.add_argument('--input-kind', choices=['normal-related'], default='normal-related')
a = ap.parse_args()
base = a.default_cohort.resolve(strict=True)
b = read(base / 'cohort.json')
assert a.input_kind=='normal-related'
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and not queue.get('source_and_corresponding_QA_frozen')
assert base.is_relative_to(ROOT/'驗證紀錄/current/native-installed-resource-profile-normal')
npath=base/'qualified/qualified.json';n=read(npath);normal_build=base;normal=b
assert b['status']=='executed_installed_resource_profile_normal_related_both_profiles'and b['owned_process']is None
assert n['cohort_sha256']==sha(base/'cohort.json')and n['recorded_owned_processes_all_terminal']and n['observed_terminal_exit_code']==0
identity=Path(normal['application'])/'applied-source-inputs.json';pins=read(identity);inputs=read(normal_build/'compiled-inputs.json')
assert len(pins)==1048 and len(inputs)==1049
assert b['source_manifest_sha256']==n['source_manifest_sha256']==sha(identity)=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert pins==read(normal_build/'source-inputs.json')
assert n['status']=='independently_qualified_installed_resource_profile_normal_related_both_profiles' and n['passed_per_profile']==19 and n['recorded_owned_processes_all_terminal']
assert not n['source_overlays_used'] and n['observed_terminal_exit_code']==0
n['entry_sha256']=sha(ROOT/'啟動現代客戶端.ps1')
source_archive=Path(n['compiled_source_archive']);assert sha(source_archive)==n['compiled_source_archive_sha256']
with zipfile.ZipFile(source_archive) as z:
    assert set(z.namelist())==set(inputs) and len(z.namelist())==1049
    for name,digest in inputs.items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==digest
fixture_path = ROOT / '驗證紀錄/current/native-teleport-wire/library-pet-fixture-map.json'
fixtures = read(fixture_path)
assert (len(fixtures['inputs']), len(fixtures['outputs']), len(fixtures['files'])) == (19,8,353)
tools = ['inventory-pinned-library-tests.py','audit-ignored-library-sources.py',
    'run-pinned-library-tests.ps1','preserve-legacy-library-reports.py']
qa = {name:sha(ROOT / '維護工具' / name) for name in tools}
assert not {k:v for k,v in os.environ.items() if v and
    (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
out = ROOT / '驗證紀錄/current/native-installed-resource-profile-library-default' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
(out / 'source-inputs.json').write_bytes(identity.read_bytes())
(out / 'compiled-inputs.json').write_bytes((normal_build / 'compiled-inputs.json').read_bytes())
(out / 'fixture-input-map.json').write_bytes(fixture_path.read_bytes())
for name in tools:
    (out / name).write_bytes((ROOT / '維護工具' / name).read_bytes())
state = dict(status='prepared_current_full_library_default_both_profiles', output=str(out),
    input_kind=a.input_kind, input_cohort=str(base),
    default_cohort=str(base) if a.input_kind == 'default116' else None,
    input_completion_sha256=sha(npath),
    default_completion_sha256=sha(npath) if a.input_kind == 'default116' else None,
    broad_default_integration_regression_claimed=False,
    normal_qualification=str(npath), normal_qualification_sha256=sha(npath),
    source_files=1048, compiled_input_files=1049, source_manifest_sha256=b['source_manifest_sha256'],
    compiled_source_archive=str(source_archive), compiled_source_archive_sha256=sha(source_archive),
    entry_script_sha256=n['entry_sha256'],normal_mains={p:v['normal']['sha256'] for p,v in n['profiles'].items()}, validation_pins=qa,
    profiles={}, owned_process=None, process_history=[], historical_pet_files=353,
    ignored_bodies_executed=False, GUI_verified=False, full_current_TCP_verified=False,
    whole_goal_complete=False, started_utc=now())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(k.GetCurrentProcess(),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
state.update(pipeline_pid=os.getpid(),pipeline_creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat())
persist = lambda: save(out / 'cohort.json', state)
env = os.environ.copy()
env.update(RUSTC=str(RUST / 'rustc.exe'), RUSTDOC=str(RUST / 'rustdoc.exe'),
    PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8',
    TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'))
env['PATH'] = str(RUST) + os.pathsep + env['PATH']
env['CARGO_TARGET_DIR']=str(PROJECT/'target/current')
for key in list(env):
    if key.startswith('LITO_'):del env[key]

def check():
    for name, h in inputs.items():
        assert sha(ROOT / name) == h, name
    for name, h in fixtures['files'].items():
        assert sha(ROOT / name) == h, name
    for name, h in qa.items():
        assert sha(ROOT / '維護工具' / name) == h, name
    assert sha(ROOT / '啟動現代客戶端.ps1') == n['entry_sha256']
    for p, info in normal['profiles'].items():
        assert sha(info['normal']['path']) == state['normal_mains'][p]

def execute(command, folder):
    folder.mkdir()
    with (folder / 'stdout.log').open('wb') as stdout, (folder / 'stderr.log').open('wb') as stderr:
        child = subprocess.Popen(command, cwd=PROJECT, env=env, stdout=stdout, stderr=stderr,
            creationflags=subprocess.CREATE_NO_WINDOW)
        times = [wintypes.FILETIME() for _ in range(4)]
        get = ctypes.WinDLL('kernel32', use_last_error=True).GetProcessTimes
        get.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        get.restype = wintypes.BOOL
        assert get(wintypes.HANDLE(int(child._handle)), *[ctypes.byref(t) for t in times])
        ticks = times[0].dwLowDateTime | times[0].dwHighDateTime << 32
        created = datetime(1601,1,1,tzinfo=timezone.utc) + timedelta(microseconds=ticks//10)
        item = dict(pid=child.pid, creation_utc=created.isoformat(), command=command, output=str(folder))
        state['owned_process'] = item
        state['process_history'].append(item)
        persist()
        code = child.wait()
        item.update(exit_code=code, finished_utc=now())
        state['owned_process'] = None
        persist()
    return code

persist()
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
try:
    for profile in ['Debug','Release']:
        check()
        d = out / profile
        d.mkdir()
        (d / 'source-inputs.json').write_bytes((out / 'source-inputs.json').read_bytes())
        state.update(status='compiling_current_full_library_test_image', active_profile=profile)
        state['profiles'][profile] = dict(directory=str(d))
        persist()
        command = [str(RUST / 'cargo.exe'),'test','--no-run','--locked','--offline','-j1',
            '--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--lib']
        if profile == 'Release':
            command.append('--release')
        code = execute(command, d / 'build')
        state['profiles'][profile]['compile_exit_code'] = code
        persist()
        assert code == 0, 'Retain compiler failure before repair.'
        messages = [json.loads(l) for l in (d / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
        artifacts = [m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']]
        assert len(artifacts)==1 and artifacts[0]['target']['name']=='lito_modern' and artifacts[0]['features']==[]
        exe = Path(artifacts[0]['executable']).resolve(strict=True)
        assert exe.is_relative_to(PROJECT / 'target/current')
        digest = sha(exe)
        archive = d / 'library-test-product.zip'
        with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
            z.write(exe,'lito_modern.exe')
        with zipfile.ZipFile(archive) as z:
            with z.open('lito_modern.exe') as f:
                assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest
        save(d / 'receipt.json',dict(status='compiled_current_full_library_test_image',exit_code=0,
            sources_unchanged=True,source_files=1048,source_manifest_sha256=state['source_manifest_sha256'],
            features=[],profile=profile,compiler_artifact_profile=artifacts[0]['profile'],build_arguments=command))
        save(d / 'product-retention.json',dict(product=str(exe),product_sha256=digest,test_target='lib',
            archive=str(archive),archive_sha256=sha(archive),archive_readback_verified=True))
        state['profiles'][profile]['product_sha256'] = digest
        persist()
        for name in ['inventory-pinned-library-tests.py','audit-ignored-library-sources.py']:
            assert execute(['python','-X','utf8',str(ROOT / '維護工具' / name),str(d)],d / (name+'-console'))==0
        check()
        state['status'] = 'running_current_full_library_default_selection'
        persist()
        code = execute(['pwsh.exe','-NoProfile','-File',str(ROOT / '維護工具/run-pinned-library-tests.ps1'),
            '-BuildRun',str(d),'-RunLabel','full-current','-PetFixtures'],d / 'runtime-console')
        result = read(d / 'runtime-full-current/receipt.json')
        state['profiles'][profile].update(runtime_helper_exit_code=code,
            counts={k:result[k] for k in ['passed','failed','ignored','filtered_out','exit_code']})
        persist()
        assert code==0 and result['status']=='passed_pinned_library_tests' and result['failed']==result['filtered_out']==0
        check()
        print(json.dumps(dict(profile=profile,counts=state['profiles'][profile]['counts']),ensure_ascii=False),flush=True)
    state.update(status='executed_both_current_full_library_default_selections',active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error))
    raise
finally:
    state['finished_utc']=now()
    persist()
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
