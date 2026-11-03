"""Run a spatial candidate only during the confirmed broad compilation phase."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,tomllib,time
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
broad=ROOT/'驗證紀錄/current/default-native-regression/20261018-033903-037-Release';normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035;source=read(normal/'cohort.json')['source_manifest_sha256']
candidate=ROOT/'維護工具/candidates/native-cone-cold/spatial.rs';candidate_sha=sha(candidate)
motion=ROOT/'驗證紀錄/current/native-cone-cold-motion/20261018-035026-100702/binding.json';assert read(motion)['status']=='primary_type5_motion_and_arrival_control_flow_bound_only'
def guard():
    r=read(broad/'receipt.json');assert r['status']=='compiling_default_targets' and not r['results'],'no candidate compilation/runtime beside timed broad tests'
    owner=r['owned_process'];pid=int(owner['pid']);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
    live=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',command],creationflags=subprocess.CREATE_NO_WINDOW).decode('utf-8-sig'));assert live['ProcessId']==pid and abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(owner['creation_utc'])).total_seconds())<.001
    for n,h in inputs.items():assert sha(ROOT/n)==h
    assert sha(candidate)==candidate_sha
    return live
observation=guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-cone-cold-spatial-candidate'/stamp;out.mkdir(parents=True);runtime=P/'target/current/native-cone-cold-spatial-candidate'/stamp;runtime.mkdir(parents=True)
(out/'motion.rs').write_bytes(candidate.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',inputs)
release=tomllib.loads((P/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release'];assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(p):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
k.GetCurrentProcess.restype=wintypes.HANDLE
class OriginalRunner:pass
original=OriginalRunner();original._handle=k.GetCurrentProcess()
direction_binding=ROOT/'驗證紀錄/current/native-cone-cold-direction/20261018-040839-795290/binding.json';assert read(direction_binding)['status']=='primary_native_direction_integer_thresholds_bound_only'
state=dict(primary_direction_binding=str(direction_binding),primary_direction_binding_sha256=sha(direction_binding),original_runner=dict(pid=os.getpid(),creation_utc=birth(original)),status='prepared_isolated_native_spatial_candidate',source_manifest_sha256=source,candidate_sha256=candidate_sha,primary_motion_binding=str(motion),primary_motion_binding_sha256=sha(motion),broad_compile_live_observation=observation,profiles={},process_history=[],owned_process=None,normal_product_registered=False,world_cell_conversion_qualified=False,timer_units_qualified=False,renderer_qualified=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(out/'receipt.json',state)
def execute(command,d):
    d.mkdir()
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=p.pid,creation_utc=birth(p),command=command,logs=str(d));state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
        job['exit_code']=p.wait();state['owned_process']=None;save(out/'receipt.json',state)
    return job['exit_code']
print('COLD_MOTION_CANDIDATE='+str(out),flush=True)
try:
    for profile in ['Debug','Release']:
        guard();d=out/profile;d.mkdir();exe=runtime/(profile.lower()+'.exe')
        command=[str(rust),str(out/'motion.rs'),'--edition=2021','--crate-name','native_cold_spatial_candidate','--test','--target','x86_64-pc-windows-msvc','-o',str(exe)]
        if profile=='Release':
            for key in ['opt-level','lto','codegen-units','strip']:command+=['-C',key+'='+str(release[key])]
        assert execute(command,d/'compile')==0,'preserve candidate compiler failure'
        pe=exe.read_bytes();e=int.from_bytes(pe[0x3c:0x40],'little');assert pe[e:e+4]==b'PE\0\0' and int.from_bytes(pe[e+4:e+6],'little')==0x8664
        guard();assert execute([str(exe),'--list'],d/'inventory')==0
        inventory=[s.split(': test')[0] for s in (d/'inventory/stdout.log').read_text(encoding='utf-8-sig').splitlines() if s.endswith(': test')];assert len(inventory)==12
        guard();assert execute([str(exe),'--test-threads=1'],d/'runtime')==0
        log=(d/'runtime/stdout.log').read_text(encoding='utf-8-sig');terminal=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log);assert terminal==[('12','0','0','0','0')]
        assert all('test '+name+' ... ok' in log for name in inventory)
        state['profiles'][profile]=dict(product=str(exe),product_sha256=sha(exe),inventory=inventory,passed=12,failed=0,ignored=0);save(out/'receipt.json',state);guard()
    state['status']='executed_both_profiles_isolated_spatial_candidate_only'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),passed_each=12,new_casts=0,renderer_qualified=False,normal_product_registered=False),ensure_ascii=False),flush=True)
