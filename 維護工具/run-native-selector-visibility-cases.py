"""Run eight ordinary normal-EXE cases, retaining each actual subprocess result."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def decode_output(path):
    raw=path.read_bytes()
    try:return raw.decode('utf-8-sig'),'utf-8-sig'
    except UnicodeDecodeError:
        text=raw.decode('cp950');assert text.encode('cp950')==raw
        return text,'cp950'
ap=argparse.ArgumentParser();ap.add_argument('preflight',type=Path);ap.add_argument('--resume',type=Path);a=ap.parse_args()
run=a.preflight.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
q=read(run/'qualified/qualified.json');assert q['observed_terminal_exit_code']==0 and q['actual_casts']==0
build_profile=read(run/'cohort.json')['profile'];assert build_profile in ['Debug','Release']
pins=read(run/'compiled-inputs.json');assert len(pins)==1029
normal=PROJECT/('target/current/x86_64-pc-windows-msvc/'+build_profile.lower()+'/LiToModern.exe')
helpers=['verify-native-selector-peers.ps1','verify-selector-peer-visibility.py','owned-peer-validation-binding.ps1']
helper_pins={n:sha(ROOT/'維護工具'/n) for n in helpers}
out=run.parent/(build_profile+'-cases-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
for n in helpers:(out/n).write_bytes((ROOT/'維護工具'/n).read_bytes())
state=dict(status=f'running_normal_{build_profile}_visibility_cases',build_profile=build_profile,output=str(out),preflight=str(run),
    preflight_qualification_sha256=sha(run/'qualified/qualified.json'),source_manifest_sha256=q['source_manifest_sha256'],
    normal_sha256=q['normal_sha256'],helper_pins=helper_pins,cases=[],owned_process=None,whole_goal_complete=False)
if a.resume:
    assert build_profile=='Debug','Historical Debug-only resume does not apply to Release'
    previous=a.resume.resolve(strict=True);assert previous.is_relative_to(run.parent)
    old=read(previous/'cohort.json');failure=read(previous/'failure-before-encoding-reader-repair/readback.json')
    assert old['status']=='failed_preserved' and old['source_manifest_sha256']==state['source_manifest_sha256']
    assert failure['original_session_id']==61582 and failure['observed_terminal_exit_code']==1
    assert sha(previous/'failure-before-encoding-reader-repair/raw.zip')==failure['raw_archive_sha256']
    assert len(old['cases'])==1 and old['cases'][0]['wrapper']['exit_code']==0 and old['helper_pins']==helper_pins
    first=old['cases'][0];folder=Path(failure['first_case']).resolve(strict=True)
    assert folder.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers')
    receipt=read(folder/'receipt.json');comparison=read(folder/'visibility-comparison.json')
    assert comparison['source_manifest_sha256']==state['source_manifest_sha256'] and comparison['receipt_sha256']==sha(folder/'receipt.json')
    assert receipt['exe_sha256']==state['normal_sha256'] and (folder/'validation-completed.json').exists()
    assert sha(folder/'visibility-comparison.json')==failure['first_case_comparison_sha256']
    text,encoding=decode_output(Path(first['output'])/'stdout.log')
    assert Path(re.findall(r'^PEER_CASE_OUTPUT=(.+)$',text,re.M)[0].strip())==folder
    first.update(case=str(folder),status='executed_and_independently_compared',stdout_encoding=encoding,
        receipt_sha256=sha(folder/'receipt.json'),comparison_sha256=sha(folder/'visibility-comparison.json'),
        actual_client_sessions=comparison['actual_client_sessions'],actual_cast_requests=comparison['actual_cast_requests'])
    state['cases']=[first];state['retained_completed_case_from_failed_batch']=failure
def persist():save(out/'cohort.json',state)
def check():
    assert sha(normal)==q['normal_sha256']
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for n,h in helper_pins.items():assert sha(ROOT/'維護工具'/n)==h,n
def identity(p,args):
    times=[wintypes.FILETIME() for _ in range(4)];k=ctypes.WinDLL('kernel32',use_last_error=True)
    k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
    assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
    tick=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
    return dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=tick//10)).isoformat(),arguments=args)
def archive(path,files):
    hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(path) as z:
        assert set(z.namelist())==set(hashes) and len(z.namelist())==len(hashes)
        for n,h in hashes.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    return dict(path=str(path),sha256=sha(path),files=hashes)
env=dict(os.environ,PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1',
    TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
check();persist();print(json.dumps(dict(output=str(out),source=state['source_manifest_sha256']),ensure_ascii=False),flush=True)
try:
    for resource in ['tw13081901-v1','lito-legacy']:
        for profile in ['detection-invisibility','detection-refused','invisibility-refused','invisibility-missing']:
            if any(v['resource']==resource and v['profile']==profile for v in state['cases']):continue
            check();label=resource+'-'+profile;d=out/label;d.mkdir()
            args=['pwsh.exe','-NoProfile','-File',str(ROOT/'維護工具/verify-native-selector-peers.ps1'),'-SkipBuild',
                '-BuildProfile',build_profile,'-ReferenceJar',str(PROJECT/'target/current/server-reference/formal-server-C02AAAE8.jar'),
                '-ResourceProfile',resource,'-ActionProfile',profile,'-VisibilityNormalInputsPath',str(run)]
            if resource=='tw13081901-v1':args+=['-CommonCatalogPath',str(PROJECT/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'),'-ExtendedItemDetails']
            with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
                p=subprocess.Popen(args,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
                job=identity(p,args);case=dict(profile=profile,resource=resource,output=str(d),wrapper=job)
                state['cases'].append(case);state['owned_process']=job;persist()
                code=p.wait();job['exit_code']=code;state['owned_process']=None;persist()
            text,encoding=decode_output(d/'stdout.log');case['stdout_encoding']=encoding
            matches=re.findall(r'^PEER_CASE_OUTPUT=(.+)$',text,re.M);assert len(matches)==1
            folder=Path(matches[0].strip()).resolve(strict=True);assert folder.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers')
            case['case']=str(folder);persist()
            if code!=0:
                files={'runner.py':out/'runner.py','cohort.json':out/'cohort.json','stdout.log':d/'stdout.log','stderr.log':d/'stderr.log'}
                for n in helpers:files['helpers/'+n]=out/n
                for p in folder.rglob('*'):
                    if p.is_file():files['case/'+p.relative_to(folder).as_posix()]=p
                ready=folder/'ready.json'
                if ready.exists():
                    # Runtime is named by this exact owned case, never by a
                    # user path supplied by the server.
                    runtime=PROJECT/'target/current/native-real-peers'/folder.name
                    for p in runtime.rglob('*'):
                        if p.is_file():files['runtime/'+p.relative_to(runtime).as_posix()]=p
                case['failure_archive']=archive(d/'failure-raw.zip',files);persist()
                raise RuntimeError('Actual case failed; preserve before repair: '+str(folder))
            check();receipt=read(folder/'receipt.json');comparison=read(folder/'visibility-comparison.json')
            assert receipt['exe_sha256']==q['normal_sha256'] and comparison['source_manifest_sha256']==state['source_manifest_sha256']
            assert comparison['receipt_sha256']==sha(folder/'receipt.json') and (folder/'validation-completed.json').exists()
            case.update(status='executed_and_independently_compared',receipt_sha256=sha(folder/'receipt.json'),
                comparison_sha256=sha(folder/'visibility-comparison.json'),actual_client_sessions=comparison['actual_client_sessions'],
                actual_cast_requests=comparison['actual_cast_requests'])
            persist();print(json.dumps(dict(case=label,status=case['status'],sessions=case['actual_client_sessions'],cast_requests=case['actual_cast_requests']),ensure_ascii=False),flush=True)
    state.update(status=f'executed_normal_{build_profile}_visibility_eight_cases',actual_client_sessions=sum(c['actual_client_sessions'] for c in state['cases']),
        actual_cast_requests=sum(c['actual_cast_requests'] for c in state['cases']))
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    state['finished_utc']=datetime.now(timezone.utc).isoformat();persist()
print(json.dumps({k:state[k] for k in ['status','actual_client_sessions','actual_cast_requests']},ensure_ascii=False),flush=True)
