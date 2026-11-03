"""Execute the staged detection regression and retain actual compiler/test evidence."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,sys,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('application',type=Path);a=ap.parse_args()
app=a.application.resolve(strict=True);assert app.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
application=read(app/'application.json');assert application['status']=='red_detection_request_test_staged_only'
assert not (app/'red-result.json').exists()
pins=read(app/'red-compiled-inputs.json');assert len(pins)==1025
for n,h in pins.items():assert sha(ROOT/n)==h
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
env=os.environ.copy();env.update(PATH=str(RUST)+os.pathsep+env['PATH'],RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),
    TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1')
command=[str(RUST/'cargo.exe'),'test','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current',
    '--message-format=json','--test','native_skills',application['red_test_name'],'--','--exact','--test-threads=1']
state=dict(status='running_red_detection_request_regression',arguments=command,actual_formal_sessions=0,whole_goal_complete=False)
(app/'red-runner.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(application=str(app),status=state['status']),ensure_ascii=False),flush=True)
code=None
try:
    with (app/'red-stdout.log').open('wb') as stdout,(app/'red-stderr.log').open('wb') as stderr:
        p=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];k=ctypes.WinDLL('kernel32',use_last_error=True)
        k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
        assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
        tick=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        state.update(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=tick//10)).isoformat())
        save(app/'red-process.json',state);code=p.wait()
    text=(app/'red-stdout.log').read_text(encoding='utf-8-sig')
    artifacts=[json.loads(l) for l in text.splitlines() if l.startswith('{')]
    tests=[m for m in artifacts if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test'] and m['target']['name']=='native_skills']
    assert len(tests)==1
    product=Path(tests[0]['executable']).resolve(strict=True);assert product.is_relative_to(PROJECT/'target/current')
    (app/'red-test.exe').write_bytes(product.read_bytes())
    assert code==101 and application['red_expected_reason'] in text
    assert 'test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured;' in text
    assert application['red_test_name'] in text
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    state.update(status='expected_detection_request_missing_reproduced',test_exit_code=code,test_product_sha256=sha(app/'red-test.exe'),
        test_product_source=str(product),source_sha256=sha(app/'red-native_skills.rs'),
        normal_mains_after={p:sha(v['path']) for p,v in application['normal_mains_before'].items()})
except BaseException as error:
    state.update(status='unexpected_red_failure_preserved',test_exit_code=code,error=repr(error));raise
finally:
    save(app/'red-result.json',state)
    names=['application.json','red-runner.py','red-process.json','red-result.json','red-compiled-inputs.json','red-native_skills.rs','red-stdout.log','red-stderr.log','red-test.exe']
    files={n:app/n for n in names if (app/n).exists()};pins={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(app/'red-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(app/'red-raw.zip') as z:
        assert set(z.namelist())==set(pins)
        for n,h in pins.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    save(app/'red-archive-readback.json',dict(archive_sha256=sha(app/'red-raw.zip'),files=pins))
print(json.dumps(dict(application=str(app),status=state['status'],test_exit_code=code),ensure_ascii=False),flush=True)
sys.exit(code)
