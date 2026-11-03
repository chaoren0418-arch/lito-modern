"""Reuse the actual source1021 library test images for both full default selections.

The compilation receipt remains the original related-scope run. No compiler or
profile override is introduced here; historical reports and pet inputs use the
existing preservation/runtime tools.
"""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess,zipfile

ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
BUILD=ROOT/'驗證紀錄/current/native-actor-green-clock/related-20261016-161430-077487'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
now=lambda:datetime.now(timezone.utc).isoformat()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
n=read(BUILD/'qualified.json');build=read(BUILD/'receipt.json');pins=read(BUILD/'source-inputs.json')
assert n['source_files']==len(pins)==1021 and n['features']==[] and build['status']=='executed_green_clock_related_scope_and_unique_entry_normal_builds'
assert n['source_manifest_sha256']=='180741A87501C13178B8521318365810B458324663F9D419758AF983099415E9'
fixtures_path=ROOT/'驗證紀錄/current/native-teleport-wire/library-pet-fixture-map.json';fixtures=read(fixtures_path)
assert len(fixtures['inputs'])==19 and len(fixtures['outputs'])==8 and len(fixtures['files'])==353
def check():
    for name,h in pins.items():assert sha(ROOT/name)==h,name
    for name,h in fixtures['files'].items():assert sha(ROOT/name)==h,name
    for profile,item in n['profiles'].items():assert sha(Path(item['path']))==item['sha256']
check()
out=ROOT/'驗證紀錄/current/native-actor-green-clock'/('full-library-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir();(out/'runner.py').write_bytes(Path(__file__).read_bytes())
tools=['inventory-pinned-library-tests.py','audit-ignored-library-sources.py','run-pinned-library-tests.ps1','preserve-legacy-library-reports.py']
qa={name:sha(ROOT/'維護工具'/name) for name in tools}
for name in tools:(out/name).write_bytes((ROOT/'維護工具'/name).read_bytes())
save(out/'source-inputs.json',pins);(out/'fixture-input-map.json').write_bytes(fixtures_path.read_bytes())
state=dict(status='prepared_default_library_runtime',output=str(out),source_files=1021,source_manifest_sha256=n['source_manifest_sha256'],
    normal_qualification=str(BUILD/'qualified.json'),normal_qualification_sha256=sha(BUILD/'qualified.json'),normal_mains={k:v['sha256'] for k,v in n['profiles'].items()},
    original_compilation_receipt=str(BUILD/'receipt.json'),original_compilation_receipt_sha256=sha(BUILD/'receipt.json'),validation_pins=qa,
    historical_pet_input_files=353,historical_pet_input_keys=19,historical_pet_output_keys=8,current_formal_pet_race_verified=False,
    profiles={},owned_process=None,process_history=[],GUI_verified=False,full_current_TCP_regression_verified=False,whole_goal_complete=False,started_utc=now())
persist=lambda:save(out/'cohort.json',state)
env=os.environ.copy();env.update(PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
def execute(command,folder):
    folder.mkdir()
    with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        process=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];get=ctypes.WinDLL('kernel32',use_last_error=True).GetProcessTimes
        get.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;get.restype=wintypes.BOOL
        assert get(wintypes.HANDLE(int(process._handle)),*[ctypes.byref(t) for t in times])
        ticks=(times[0].dwHighDateTime<<32)|times[0].dwLowDateTime
        created=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
        item=dict(pid=process.pid,creation_utc=created.isoformat(),args=command,log_directory=str(folder))
        state['owned_process']=item;state['process_history'].append(item);persist()
        code=process.wait();item.update(exit_code=code,finished_utc=now());state['owned_process']=None;persist()
    return code
persist();print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
try:
    for profile in ['Debug','Release']:
        check()
        for name,h in qa.items():assert sha(ROOT/'維護工具'/name)==h,name
        d=out/profile;d.mkdir();(d/'source-inputs.json').write_bytes((BUILD/'source-inputs.json').read_bytes())
        product=read(BUILD/profile/'products.json')['lito_modern'];assert sha(Path(product['path']))==product['sha256']
        archive=BUILD/profile/'test-products.zip';assert sha(archive)==build['profiles'][profile]['test_archive']['sha256']
        with zipfile.ZipFile(archive) as z:
            with z.open('lito_modern.exe') as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==product['sha256']
        messages=[json.loads(line) for line in (BUILD/profile/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
        artifact=next(m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable')
                      and Path(m['executable']).resolve()==Path(product['path']).resolve() and m['profile']['test'])
        assert artifact['features']==[]
        save(d/'receipt.json',dict(status='retained_existing_compiled_library_for_default_runtime',exit_code=build['profiles'][profile]['compile_exit_code'],
            sources_unchanged=True,source_files=1021,source_manifest_sha256=n['source_manifest_sha256'],features=[],profile=profile,
            original_compilation_directory=str(BUILD/profile),original_compilation_receipt_sha256=sha(BUILD/'receipt.json'),compiler_artifact_profile=artifact['profile']))
        save(d/'product-retention.json',dict(product=product['path'],product_sha256=product['sha256'],test_target='lib',archive=str(archive),
            archive_sha256=sha(archive),archive_readback_verified=True,retained_archive_member='lito_modern.exe',no_new_compilation=True))
        state.update(status='inventorying_full_library_default',active_profile=profile);state['profiles'][profile]=dict(directory=str(d),product_sha256=product['sha256']);persist()
        for name in ['inventory-pinned-library-tests.py','audit-ignored-library-sources.py']:
            code=execute(['python','-X','utf8',str(ROOT/'維護工具'/name),str(d)],d/(name+'-console'));assert code==0,name
        state['status']='running_full_library_default';persist()
        code=execute(['pwsh.exe','-NoProfile','-File',str(ROOT/'維護工具/run-pinned-library-tests.ps1'),'-BuildRun',str(d),'-RunLabel','full-current','-PetFixtures'],d/'runtime-console')
        result=read(d/'runtime-full-current/receipt.json');state['profiles'][profile].update(runtime_exit_code=code,counts={k:result[k] for k in ['passed','failed','ignored','filtered_out']});persist()
        assert code==0 and result['status']=='passed_pinned_library_tests' and result['failed']==result['filtered_out']==0,'Preserve full-default failures.'
        check();print(json.dumps(dict(profile=profile,counts=state['profiles'][profile]['counts']),ensure_ascii=False),flush=True)
    state.update(status='executed_both_source1021_full_library_default_selections',active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    state['finished_utc']=now();persist()
print(json.dumps(dict(output=str(out),status=state['status'],profiles=state['profiles']),ensure_ascii=False),flush=True)
