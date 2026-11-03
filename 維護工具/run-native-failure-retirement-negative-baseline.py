"""Demonstrate the existing failure with only two added retirement assertions.

Temporary test-only overlay; restore original source and original library PE in
finally. No current normal-product or full-library qualification is granted.
"""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
now=lambda:datetime.now(timezone.utc).isoformat()
ap=argparse.ArgumentParser();ap.add_argument('candidate',type=Path);a=ap.parse_args();candidate=a.candidate.resolve(strict=True)
assert candidate.is_relative_to(R/'維護工具/candidates')
c=read(candidate/'candidate.json');w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256']
assert source==c['before_source_manifest_sha256']=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
controller=w['current_native_catalog_controller_qualification'];assert controller['source_manifest_sha256']==source and controller['qualified'] and controller['actual_sessions']==96
assert sha(controller['qualification'])==controller['qualification_sha256']
normal=read(w['current_native_installed_resource_profile_normal_qualification']['qualification']);assert normal['source_manifest_sha256']==source and sha(normal['compiled_source_archive'])==normal['compiled_source_archive_sha256']
base=Path(w['current_native_catalog_library']['run']);inputs=read(base/'compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
for n,h in c['before_files'].items():assert sha(R/n)==h
test_name='LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs';original=(R/test_name).read_bytes();payload=(candidate/Path(test_name).name).read_bytes();assert hashlib.sha256(payload).hexdigest().upper()==c['after_files'][test_name]
retention=read(base/'Debug/product-retention.json');old_exe=Path(retention['product']);assert old_exe.is_relative_to(P/'target/current') and sha(old_exe)==retention['product_sha256'] and sha(retention['archive'])==retention['archive_sha256']
with zipfile.ZipFile(retention['archive'])as z:
    assert len(z.namelist())==1
    old_member=z.namelist()[0]
    with z.open(old_member)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==retention['product_sha256']
names=['modern_lito::native_login::rental_return_tests::'+n for n in ['unacknowledged_return_timeout_cannot_open_login_or_keep_credentials','missing_source_return_timeout_cannot_connect_an_owned_login_peer']]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32);return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
out=V/'native-failure-retirement-negative-baseline'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'original-test.rs').write_bytes(original);(out/'added-retirement-assertions.rs').write_bytes(payload)
overlay=dict(inputs);overlay[test_name]=hashlib.sha256(payload).hexdigest().upper();save(out/'before-compiled-inputs.json',inputs);save(out/'overlay-compiled-inputs.json',overlay)
state=dict(status='prepared_test_only_counterexample',source_manifest_sha256=source,runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),candidate=str(candidate),candidate_sha256=sha(candidate/'candidate.json'),changed_test_source=test_name,runtime_product_sources_unchanged=True,test_only_overlay=True,normal_products_modified=False,expected_tests=names,expected_failed=2,expected_Rust_exit_code=101,original_product_retention=retention,process_history=[],owned_process=None,restored=False,GUI_verified=False,whole_goal_complete=False)
state.update(original_compiled_source_archive=normal['compiled_source_archive'],original_compiled_source_archive_sha256=normal['compiled_source_archive_sha256'])
save(out/'cohort.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
assert not {n:v for n,v in os.environ.items()if v and(n.startswith('CARGO_PROFILE_')or n in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
env={n:v for n,v in os.environ.items()if not n.startswith('LITO_')};env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),RUSTC=str(rust/'rustc.exe'),RUSTDOC=str(rust/'rustdoc.exe'));env['PATH']=str(rust)+os.pathsep+env['PATH']
def guard():
    for n,h in overlay.items():assert sha(R/n)==h,n
def execute(command,label):
    guard();d=out/label;d.mkdir()
    with(d/'stdout.log').open('wb')as stdout,(d/'stderr.log').open('wb')as stderr:
        child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=label,exit_code=None);state['process_history'].append(row);state['owned_process']=row;save(out/'cohort.json',state)
        try:code=child.wait()
        finally:
            if child.poll()is None:
                assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc'];child.terminate();child.wait(timeout=3);row['cleanup_stopped_original_handle']=True
            row['exit_code']=child.returncode;state['owned_process']=None;save(out/'cohort.json',state)
    guard();return code
try:
    (R/test_name).write_bytes(payload);guard();state['status']='compiling_test_only_counterexample';save(out/'cohort.json',state)
    assert execute([str(rust/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--lib'],'build')==0
    messages=[json.loads(l)for l in(out/'build/stdout.log').read_text('utf-8-sig').splitlines()if l.startswith('{')]
    artifacts=[m for m in messages if m.get('reason')=='compiler-artifact'and m.get('executable')and m['profile']['test']];assert len(artifacts)==1 and artifacts[0]['features']==[]and artifacts[0]['target']['name']=='lito_modern'
    exe=Path(artifacts[0]['executable']).resolve(strict=True);assert exe==old_exe.resolve()
    state.update(status='running_expected_two_retirement_counterexamples',product=str(exe),product_sha256=sha(exe));save(out/'cohort.json',state)
    with zipfile.ZipFile(out/'test-product.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:z.write(exe,exe.name)
    code=execute([str(exe),*names,'--exact','--test-threads=1'],'tests');text=(out/'tests/stdout.log').read_text('utf-8-sig')
    found=re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$',text,re.M)
    assert code==101 and sorted(found)==sorted((n,'FAILED')for n in names)
    assert re.findall(r'test result: FAILED\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('0','2','0','0','2460')]
    assert text.count('assertion failed: c.snapshot().joined_room.is_none() && c.snapshot().rooms.is_empty()')==2
    state.update(status='observed_expected_two_retirement_counterexamples_pending_independent_readback',actual_Rust_exit_code=code,counts=dict(passed=0,failed=2,ignored=0,filtered_out=2460),actual_formal_sessions=0)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    assert sha(R/test_name)==overlay[test_name]
    (R/test_name).write_bytes(original)
    with zipfile.ZipFile(retention['archive'])as z:
        old_bytes=z.read(old_member);assert hashlib.sha256(old_bytes).hexdigest().upper()==retention['product_sha256'];old_exe.write_bytes(old_bytes)
    assert all(sha(R/n)==h for n,h in inputs.items()) and sha(old_exe)==retention['product_sha256']
    state.update(restored=True,original_library_product_restored=True,finished_utc=now());save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],actual_Rust_exit_code=state['actual_Rust_exit_code'],original_sources_and_PE_restored=True),ensure_ascii=False),flush=True)
