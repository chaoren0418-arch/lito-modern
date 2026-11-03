"""Extend the exact isolated library qualification without recompiling its body."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
ap=argparse.ArgumentParser();ap.add_argument('--base',type=Path,default=V/'native33199-ordinary-admission-v2-ready/20261018-145825-685763');a=ap.parse_args();base=a.base.resolve(strict=True);assert base.is_relative_to(V/'native33199-ordinary-admission-v2-ready')
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
q=read(base/'qualified.json');b=read(base/'receipt.json');assert q['scope']['passed']==4 and q['recorded_owned_all_terminal']
pins=read(V/'native-scoped-catalog-normal/validation-20261018-132210-585372/compiled-inputs.json')
def guard():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for n,pin in b['source_overlays'].items():assert sha(base/n)==pin['sha256'],n
    assert sha(b['diagnostic_library'])==b['diagnostic_library_sha256'] and sha(b['product'])==b['product_sha256']
guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native33199-admission-lifecycle'/stamp;out.mkdir(parents=True);runtime=P/'target/current/native33199-admission-lifecycle'/stamp;runtime.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'base-receipt.json').write_bytes((base/'receipt.json').read_bytes());(out/'base-qualified.json').write_bytes((base/'qualified.json').read_bytes())
life=ROOT/'維護工具/candidates/native33199-ordinary-admission-v2-ready/lifecycle-regressions.rs';(out/life.name).write_bytes(life.read_bytes());(out/'lifecycle.rs').write_bytes((base/'tests.rs').read_bytes()+b'\n'+life.read_bytes())
original=b['process_history'][1]['arguments'];assert original[1]==str(base/'tests.rs');commands=[];products={}
for target in ['lifecycle','native_animation','native_combat']:
    path=out/(target+'.rs')
    if target!='lifecycle':
        source=P/'tests'/(target+'.rs');text=source.read_text('utf-8-sig');text=re.sub(r'include_bytes!\(\s*"([^"\n]+)"\s*\)',lambda m:'include_bytes!('+json.dumps((source.parent/m[1]).resolve(strict=True).as_posix(),ensure_ascii=False)+')',text);path.write_text(text,encoding='utf-8',newline='\n');(out/(target+'.original.rs')).write_bytes(source.read_bytes())
    exe=runtime/(target+'.exe');args=original.copy();args[1]=str(path);args[args.index('-o')+1]=str(exe);args[args.index('--crate-name')+1]='admission_'+target
    products[target]={'path':str(exe),'source_sha256':sha(path)};commands.append((target+'-compile',args));commands.append((target+'-test',[str(exe)]+(['admission_lifecycle_'] if target=='lifecycle' else [])+['--nocapture','--test-threads=1']))
products['source-owner']={'path':b['product'],'sha256':b['product_sha256']};commands.append(('source-owner-test',[b['product'],'--skip','ordinary_','--nocapture','--test-threads=1']))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32);return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(profile=b.get('profile','Debug'),status='running_exact_candidate_lifecycle_and_animation_regressions',base=str(base),base_qualification_sha256=sha(base/'qualified.json'),source_manifest_sha256=b['source_manifest_sha256'],diagnostic_library=b['diagnostic_library'],diagnostic_library_sha256=b['diagnostic_library_sha256'],products=products,process_history=[],owned_process=None,runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),normal_source_modified=False,physical_audio_verified=False,Release_verified=False,valid_second_resource_source_verified=False,classic_terrain_reload_verified=False,whole_goal_complete=False)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),LITO_AUDIO_ABA_OBSERVATION=str(out/'observation.json'))
print('ADMISSION_LIFECYCLE='+str(out),flush=True)
try:
    for label,args in commands:
        guard()
        with(out/(label+'.stdout.txt')).open('wb')as stdout,(out/(label+'.stderr.txt')).open('wb')as stderr:
            child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=args);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=240 if label=='source-owner-test' else 180)
            except subprocess.TimeoutExpired:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];child.terminate();job['exit_code']=child.wait();raise
            state['owned_process']=None;save(out/'receipt.json',state);assert job['exit_code']==0,(label,job['exit_code'])
        if label.endswith('-compile'):
            target=label.removesuffix('-compile');products[target]['sha256']=sha(products[target]['path'])
        else:
            text=(out/(label+'.stdout.txt')).read_text('utf-8-sig');m=re.search(r'test result: ok\. (\d+) passed; 0 failed; (\d+) ignored',text);assert m and int(m[2])==0;state.setdefault('results',{})[label]={'passed':int(m[1]),'ignored':0};print(label+': '+m[0],flush=True)
    assert state['results']['lifecycle-test']['passed']==3 and state['results']['source-owner-test']['passed']==12
    guard();state.update(status='candidate_lifecycle_original_animation_combat_and_source_owner_passed',passed=sum(v['passed']for v in state['results'].values()),failed=0,ignored=0)
except BaseException as e:state.update(status='failed_preserved',error=repr(e));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(run=str(out),passed=state['passed'],normal_source_modified=False),ensure_ascii=False))
