"""Run the ten current native77 ignored resource cases from qualified library PEs.

This does not execute historical TLS/SQL tests or grant GUI/audio-device scope.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, re, subprocess, zipfile

R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';P=R/'LiTo381-Modern'
SOURCE='A9FE9AD547E79DC1E0C97DC01CB6C20D0119477788E193524F5B2DFB96DD4B4E'
PREFIX='modern_lito::native_disintegrate_tests::native77_probe::'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
now=lambda:datetime.now(timezone.utc).isoformat()
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(h):
    t=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(x)for x in t])
    ticks=t[0].dwLowDateTime|(t[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
ap=argparse.ArgumentParser();ap.add_argument('library',type=Path);a=ap.parse_args()
base=a.library.resolve(strict=True);assert base.is_relative_to(V/'native-initial-source-confirmation-library-default')
completion=read(base/'completion-observation.json');queue=read(R/'工作佇列.json')
assert completion['source_manifest_sha256']==SOURCE and completion['observed_terminal_exit_code']==0
assert completion['recorded_original_owned_processes_all_terminal']
assert queue['current_full_default_library_verified'] and queue['current_native_catalog_library']['run']==str(base)
assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
inputs=read(base/'compiled-inputs.json');assert len(inputs)==1048
def check():
    for n,h in inputs.items():assert sha(R/n)==h,n
    assert sha(base/'completion-observation.json')==state['library_completion_sha256']
out=V/'native-initial-source-confirmation-library-resource-optin'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True);(out/'runner.py').write_bytes(Path(__file__).read_bytes())
state=dict(status='preparing_explicit_native77_resource_optin',source_manifest_sha256=SOURCE,
    library=str(base),library_completion_sha256=sha(base/'completion-observation.json'),
    pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),profiles={},owned_process=None,
    expected_tests_per_profile=10,default_library_results_changed=False,historical_TLS_SQL_executed=False,
    resources_read_only=True,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
env=os.environ.copy()
for n in list(env):
    if n.startswith('LITO_'):del env[n]
env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
try:
    for profile in ['Debug','Release']:
        check();d=base/profile;qp=d/'full-default-qualified/qualified.json';q=read(qp)
        assert q['source_manifest_sha256']==SOURCE and q['failed']==q['Rust_exit_code']==0
        product=read(d/'product-retention.json');exe=Path(product['product']).resolve(strict=True)
        assert exe.is_relative_to(P/'target/current') and sha(exe)==product['product_sha256']==q['actual_test_sha256']
        audit=read(d/'inventory-audit/ignored-source-reasons.json')
        selected={x['test']:x for x in audit['tests'] if x['test'].startswith(PREFIX)}
        assert len(selected)==10 and all(x['source_mapping']=='unique_source_attribute' and not x['body_executed'] for x in selected.values())
        for row in selected.values():
            assert len(row['candidates'])==1
            src=row['candidates'][0];assert sha(R/src['source'])==src['source_sha256']
            assert src['reason'].startswith('read-only actual TW ')
        target=out/profile;target.mkdir();command=[str(exe),PREFIX,'--ignored','--test-threads=1']
        with(target/'tests.log').open('wb')as stdout,(target/'stderr.log').open('wb')as stderr:
            child=subprocess.Popen(command,cwd=target,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,
                product_sha256=sha(exe),qualification=str(qp),qualification_sha256=sha(qp),
                expected_tests=selected,exit_code=None)
            state.update(status='running_explicit_native77_resource_optin',active_profile=profile,owned_process=row)
            state['profiles'][profile]=row;save(out/'cohort.json',state)
            code=child.wait();row.update(exit_code=code,finished_utc=now());state['owned_process']=None
        save(out/'cohort.json',state)
        log=(target/'tests.log').read_text('utf-8-sig')
        results=re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$',log,re.M)
        assert len(results)==10 and {n for n,_ in results}==set(selected)
        assert all(result=='ok'for _,result in results) and code==0
        totals=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
        assert len(totals)==1 and totals[0][:5]==('ok','10','0','0','0')
        row.update(passed=10,failed=0,ignored=0,filtered_out=int(totals[0][5]),
            results=dict(results),stdout_sha256=sha(target/'tests.log'),stderr_sha256=sha(target/'stderr.log'))
        assert row['filtered_out']==q['inventory']-10
        check();save(out/'cohort.json',state)
        print(json.dumps(dict(profile=profile,passed=10,failed=0,ignored=0),ensure_ascii=False),flush=True)
    state.update(status='executed_both_current_native77_resource_optin_selections',active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    state['finished_utc']=now();save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
