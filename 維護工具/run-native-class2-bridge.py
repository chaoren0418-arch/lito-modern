"""Isolated candidate lifecycle checks against the qualified environment State."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, subprocess
ROOT=Path(__file__).resolve().parents[1]
def read(p): return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v): p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--compile',type=Path,required=True);a=ap.parse_args()
compiled=a.compile.resolve(strict=True)
assert compiled.is_relative_to(ROOT/'驗證紀錄/current/native-class2-bridge')
c=read(compiled/'receipt.json');assert c['status']=='compiled_current_production_library_ambient_candidate'
broad=ROOT/'驗證紀錄/current/default-native-regression/20261017-221209-558-Release/receipt.json'
assert read(broad)['status']=='compiling_default_targets', 'no candidate runtime beside timed broad cases'
exe=Path(c['product']);assert sha(exe)==c['product_sha256']
for name,digest in read(compiled/'compiled-inputs.json').items(): assert sha(ROOT/name)==digest,name
assert sha(compiled/'native_ambient_override.rs')==c['candidate_module_sha256']
assert sha(compiled/'probe.rs')==c['compiled_probe_sha256']
assert sha(compiled/'native_ambient_bridge.rs')==c['bridge_source']['sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=compiled.parent/('run-'+stamp);out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
state=dict(status='executing_candidate_component_checks',compile=str(compiled),compile_receipt_sha256=sha(compiled/'receipt.json'),
    source_manifest_sha256=c['source_manifest_sha256'],candidate_module_sha256=c['candidate_module_sha256'],
    product=str(exe),product_sha256=sha(exe),profile=c['profile'],process_history=[],
    actual_sessions=0,real_casts_verified=False,renderer_verified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'receipt.json',state);print('AMBIENT_CANDIDATE_RUNTIME='+str(out),flush=True)
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
for name,args in [('list',['--list']),('checks',['--test-threads=1','--nocapture'])]:
    assert read(broad)['status']=='compiling_default_targets'
    with (out/(name+'.stdout.log')).open('wb') as stdout,(out/(name+'.stderr.log')).open('wb') as stderr:
        child=subprocess.Popen([str(exe),*args],cwd=ROOT,env=dict(os.environ,TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch')),stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        ts=[wintypes.FILETIME() for _ in range(4)]
        assert k.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in ts])
        ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
        record=dict(name=name,pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=args)
        state['process_history'].append(record);save(out/'receipt.json',state)
        record['exit_code']=child.wait(timeout=60);save(out/'receipt.json',state)
    if record['exit_code']:
        state['status']='failed_preserved';save(out/'receipt.json',state)
        print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
        raise SystemExit(record['exit_code'])
text=(out/'checks.stdout.log').read_text(encoding='utf-8')
assert 'test result: ok. 8 passed; 0 failed; 0 ignored' in text
assert sha(exe)==c['product_sha256']
for name,digest in read(compiled/'compiled-inputs.json').items(): assert sha(ROOT/name)==digest,name
state.update(status='executed_candidate_component_checks',passed=8,failed=0,ignored=0,
    raw_hashes={p.name:sha(p) for p in out.glob('*.log')})
save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],passed=8),ensure_ascii=False),flush=True)
