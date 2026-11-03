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
candidate=ROOT/'維護工具/candidates/native-cone-cold/cone_cold_assets.rs';candidate_sha=sha(candidate)
motion=ROOT/'驗證紀錄/current/native-cone-cold-motion/20261018-035026-100702/binding.json';assert read(motion)['status']=='primary_type5_motion_and_arrival_control_flow_bound_only'
def guard():
    r=read(broad/'receipt.json');assert r['status']=='compiling_default_targets' and not r['results'],'no candidate compilation/runtime beside timed broad tests'
    owner=r['owned_process'];pid=int(owner['pid']);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
    live=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',command],creationflags=subprocess.CREATE_NO_WINDOW).decode('utf-8-sig'));assert live['ProcessId']==pid and abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(owner['creation_utc'])).total_seconds())<.001
    for n,h in inputs.items():assert sha(ROOT/n)==h
    assert sha(candidate)==candidate_sha
    return live
observation=guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-cone-cold-assets-typecheck'/stamp;out.mkdir(parents=True);runtime=P/'target/current/native-cone-cold-assets-typecheck'/stamp;runtime.mkdir(parents=True)
(out/'cone_cold_assets.rs').write_bytes(candidate.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',inputs)
release=tomllib.loads((P/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release'];assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(p):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
k.GetCurrentProcess.restype=wintypes.HANDLE
class OriginalRunner:pass
original=OriginalRunner();original._handle=k.GetCurrentProcess()
signed_binding=ROOT/'驗證紀錄/current/native-cone-cold-signed-spatial/20261018-041923-899485/binding.json';assert read(signed_binding)['signed_vector_shift_before_idiv']
direction_binding=ROOT/'驗證紀錄/current/native-cone-cold-direction/20261018-040839-795290/binding.json';assert read(direction_binding)['status']=='primary_native_direction_integer_thresholds_bound_only'
state=dict(primary_signed_binding=str(signed_binding),primary_signed_binding_sha256=sha(signed_binding),primary_direction_binding=str(direction_binding),primary_direction_binding_sha256=sha(direction_binding),original_runner=dict(pid=os.getpid(),creation_utc=birth(original)),status='prepared_isolated_native_spatial_candidate',source_manifest_sha256=source,candidate_sha256=candidate_sha,primary_motion_binding=str(motion),primary_motion_binding_sha256=sha(motion),broad_compile_live_observation=observation,profiles={},process_history=[],owned_process=None,normal_product_registered=False,world_cell_conversion_qualified=False,timer_units_qualified=False,renderer_qualified=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(out/'receipt.json',state)
def execute(command,d):
    d.mkdir()
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=p.pid,creation_utc=birth(p),command=command,logs=str(d));state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
        job['exit_code']=p.wait();state['owned_process']=None;save(out/'receipt.json',state)
    return job['exit_code']
print('COLD_ASSETS_TYPECHECK='+str(out),flush=True)
try:
    for profile in ['Debug','Release']:
        guard();d=out/profile;d.mkdir()
        libs={};native=set()
        for line in (normal/profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
            if not line.startswith('{'):continue
            j=json.loads(line)
            if j.get('reason')=='build-script-executed':native.update(j.get('linked_paths',[]))
            if j.get('reason')=='compiler-artifact':
                for path in j['filenames']:
                    if path.endswith('.rlib'):
                        name=j['target']['name'].replace('-','_')
                        if name not in libs or '/x86_64-pc-windows-msvc/' in path.replace(chr(92),'/'):libs[name]=path
        assert {'lito_modern','anyhow','sha2'}<=libs.keys()
        pins={name:dict(path=path,sha256=sha(path)) for name,path in libs.items()}
        wrapper=out/(profile.lower()+'_probe.rs')
        wrapper.write_text('pub use lito_modern::modern_lito::{combat_animation,combat_audio,resources,sprite,sprite_actions,teleport_effect,tw_action_catalog};'+chr(10)+'#[path = "cone_cold_assets.rs"] pub mod cone_cold_assets;'+chr(10),encoding='utf-8')
        artifact=runtime/(profile.lower()+'.rmeta')
        command=[str(rust),str(wrapper),'--edition=2021','--crate-name','native_cold_assets_candidate','--crate-type','lib','--emit=metadata','--target','x86_64-pc-windows-msvc','-L','dependency='+str(P/('target/current/x86_64-pc-windows-msvc/'+profile.lower()+'/deps')),'-L','dependency='+str(P/('target/current/'+profile.lower()+'/deps')),'-o',str(artifact)]
        if profile=='Release':
            for key in ['opt-level','lto','codegen-units','strip']:command+=['-C',key+'='+str(release[key])]
        for name,path in sorted(libs.items()):command+=['--extern',name+'='+path]
        for path in sorted(native):command+=['-L',path]
        env.update(CARGO_MANIFEST_DIR=str(P))
        assert execute(command,d/'compile')==0,'preserve typecheck failure'
        assert artifact.is_file() and artifact.stat().st_size>0
        for pin in pins.values():assert sha(pin['path'])==pin['sha256']
        state['profiles'][profile]=dict(artifact=str(artifact),artifact_sha256=sha(artifact),library_pins=pins,typecheck_exit_code=0,resource_runtime_executed=False)
        save(out/'receipt.json',state);guard()
    state.update(status='typechecked_isolated_assets_candidate_both_profiles_only',resource_bank_loaded=False,resource_pack_read=False,audio_played=False,new_renders=0)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),typechecked_profiles=list(state['profiles']),resource_runtime_executed=False,normal_product_registered=False),ensure_ascii=False),flush=True)
