"""Build the corrected test-only TX retirement assertion and rerun full defaults.

Source retention is the original source1021 archive plus the one changed cfg-test
file. Normal builds must retain exactly the previously qualified EXE bytes.
"""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,zipfile

ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
BASE=ROOT/'驗證紀錄/current/native-actor-green-clock/related-20261016-161430-077487'
FAILURE=ROOT/'驗證紀錄/current/native-actor-green-clock/full-library-20261016-170854-085092'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
CHANGED='LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs'
GROUP='modern_lito::native_login::teleport_wire_tests::'
TARGET=GROUP+'malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
now=lambda:datetime.now(timezone.utc).isoformat()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
n=read(BASE/'qualified.json');before=read(BASE/'source-inputs.json');pins={name:sha(ROOT/name) for name in before}
assert len(pins)==1021 and [name for name in pins if pins[name]!=before[name]]==[CHANGED]
assert read(FAILURE/'failure-readback.json')['failed']==1
assert before[CHANGED]==sha(FAILURE/'native_teleport_controller_tests-before.rs')
assert '#[cfg(any(test, feature = "teleport-wire-test"))]' in (PROJECT/'src/modern_lito/native_login.rs').read_text(encoding='utf-8-sig')
fixtures_path=ROOT/'驗證紀錄/current/native-teleport-wire/library-pet-fixture-map.json';fixtures=read(fixtures_path)
assert (len(fixtures['inputs']),len(fixtures['outputs']),len(fixtures['files']))==(19,8,353)
def check():
    for name,h in pins.items():assert sha(ROOT/name)==h,name
    for name,h in fixtures['files'].items():assert sha(ROOT/name)==h,name
    for v in n['profiles'].values():assert sha(Path(v['path']))==v['sha256']
check()
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
out=ROOT/'驗證紀錄/current/native-teleport-wire'/('TX-retirement-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir();(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'source-inputs.json',pins)
(out/'fixture-input-map.json').write_bytes(fixtures_path.read_bytes())
with zipfile.ZipFile(out/'source-delta.zip','x',zipfile.ZIP_DEFLATED) as z:z.write(ROOT/CHANGED,CHANGED)
with zipfile.ZipFile(out/'source-delta.zip') as z:assert hashlib.sha256(z.read(CHANGED)).hexdigest().upper()==pins[CHANGED]
state=dict(status='prepared_test_only_TX_retirement_correction',output=str(out),source_files=1021,source_manifest_sha256=sha(out/'source-inputs.json'),
    changed_source=CHANGED,before_source_sha256=before[CHANGED],after_source_sha256=pins[CHANGED],
    source_retention=dict(base_archive=str(BASE/'compiled-source.zip'),base_archive_sha256=sha(BASE/'compiled-source.zip'),base_manifest=str(BASE/'source-inputs.json'),base_manifest_sha256=sha(BASE/'source-inputs.json'),delta_archive=str(out/'source-delta.zip'),delta_archive_sha256=sha(out/'source-delta.zip')),
    original_failure=str(FAILURE/'failure-readback.json'),normal_qualification=str(BASE/'qualified.json'),normal_mains={k:v['sha256'] for k,v in n['profiles'].items()},
    normal_builds={},profiles={},owned_process=None,process_history=[],historical_pet_files=353,GUI_verified=False,full_current_TCP_verified=False,whole_goal_complete=False,started_utc=now())
tools=['inventory-pinned-library-tests.py','audit-ignored-library-sources.py','run-pinned-library-tests.ps1','preserve-legacy-library-reports.py']
state['validation_pins']={name:sha(ROOT/'維護工具'/name) for name in tools}
for name in tools:(out/name).write_bytes((ROOT/'維護工具'/name).read_bytes())
persist=lambda:save(out/'cohort.json',state)
env=os.environ.copy();env.update(RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
env['PATH']=str(RUST)+os.pathsep+env['PATH']
def execute(command,folder):
    folder.mkdir(parents=True,exist_ok=False)
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];get=ctypes.WinDLL('kernel32',use_last_error=True).GetProcessTimes
        get.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;get.restype=wintypes.BOOL
        assert get(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(v) for v in times])
        ticks=(times[0].dwHighDateTime<<32)|times[0].dwLowDateTime
        created=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
        item=dict(pid=child.pid,creation_utc=created.isoformat(),args=command,log_directory=str(folder));state['owned_process']=item;state['process_history'].append(item);persist()
        code=child.wait();item.update(exit_code=code,finished_utc=now());state['owned_process']=None;persist()
    return code
def summary(log):
    matches=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log.read_text(encoding='utf-8-sig'))
    assert len(matches)==1;status,*values=matches[0];return dict(status=status,**dict(zip(['passed','failed','ignored','measured','filtered'],map(int,values))))
persist();print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
try:
    for profile in ['Debug','Release']:
        check();state.update(status='confirming_unique_entry_normal_bytes',active_profile=profile);persist()
        dep=PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.d'
        assert Path(CHANGED).name not in dep.read_text(encoding='utf-8-sig')
        folder=out/('normal-'+profile);command=['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile]
        code=execute(command,folder);assert code==0
        (folder/'entry-build.log').write_bytes((ROOT/'驗證紀錄/current'/('build-release.log' if profile=='Release' else 'build.log')).read_bytes())
        (folder/'normal-dependencies.d').write_bytes(dep.read_bytes());assert Path(CHANGED).name not in dep.read_text(encoding='utf-8-sig')
        assert sha(Path(n['profiles'][profile]['path']))==n['profiles'][profile]['sha256']
        state['normal_builds'][profile]=dict(exit_code=0,sha256=n['profiles'][profile]['sha256'],changed_test_excluded_from_normal_depfile=True,build_arguments=command);persist()
    for profile in ['Debug','Release']:
        check();d=out/profile;d.mkdir();(d/'source-inputs.json').write_bytes((out/'source-inputs.json').read_bytes())
        state.update(status='compiling_corrected_library_tests',active_profile=profile);state['profiles'][profile]=dict(directory=str(d));persist()
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--lib']
        if profile=='Release':command.append('--release')
        code=execute(command,d/'build');assert code==0,'Preserve compiler failure.'
        messages=[json.loads(line) for line in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
        rows=[m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']]
        assert len(rows)==1 and rows[0]['target']['name']=='lito_modern' and rows[0]['features']==[]
        exe=Path(rows[0]['executable']).resolve(strict=True);assert exe.is_relative_to(PROJECT/'target/current');product_hash=sha(exe)
        with zipfile.ZipFile(d/'library-test-product.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:z.write(exe,'lito_modern.exe')
        with zipfile.ZipFile(d/'library-test-product.zip') as z:
            with z.open('lito_modern.exe') as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==product_hash
        save(d/'receipt.json',dict(status='compiled_corrected_test_only_library',exit_code=0,sources_unchanged=True,source_files=1021,source_manifest_sha256=state['source_manifest_sha256'],features=[],profile=profile,compiler_artifact_profile=rows[0]['profile'],build_arguments=command))
        save(d/'product-retention.json',dict(product=str(exe),product_sha256=product_hash,test_target='lib',archive=str(d/'library-test-product.zip'),archive_sha256=sha(d/'library-test-product.zip'),archive_readback_verified=True))
        state['profiles'][profile]['product_sha256']=product_hash;persist()
        for tool in tools:
            assert sha(ROOT/'維護工具'/tool)==state['validation_pins'][tool],tool
        for tool in ['inventory-pinned-library-tests.py','audit-ignored-library-sources.py']:
            assert execute(['python','-X','utf8',str(ROOT/'維護工具'/tool),str(d)],d/(tool+'-console'))==0
        for label,filters,count in [('malformed-target',['--exact',TARGET],1),('all-six-wire-groups',[GROUP],6)]:
            state.update(status='running_corrected_TX_retirement_scope',active_group=label);persist()
            assert execute([str(exe),*filters,'--test-threads=1'],d/label)==0
            result=summary(d/label/'stdout.log');assert result['status']=='ok' and result['passed']==count and result['failed']==result['ignored']==result['measured']==0
            state['profiles'][profile][label]=result;persist();print(profile+' '+label+' passed',flush=True)
        state.update(status='running_corrected_full_library_default',active_group=None);persist()
        code=execute(['pwsh.exe','-NoProfile','-File',str(ROOT/'維護工具/run-pinned-library-tests.ps1'),'-BuildRun',str(d),'-RunLabel','full-current','-PetFixtures'],d/'runtime-console')
        result=read(d/'runtime-full-current/receipt.json');state['profiles'][profile]['full_default']={k:result[k] for k in ['passed','failed','ignored','filtered_out','exit_code']};persist()
        assert code==0 and result['status']=='passed_pinned_library_tests' and result['failed']==result['filtered_out']==0,'Preserve full-default failures.'
        check();print(json.dumps(dict(profile=profile,full_default=state['profiles'][profile]['full_default']),ensure_ascii=False),flush=True)
    state.update(status='executed_corrected_TX_retirement_and_both_full_default_libraries',active_profile=None,active_group=None)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    state['finished_utc']=now();persist()
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
