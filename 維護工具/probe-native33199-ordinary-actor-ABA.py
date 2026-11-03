"""Resource-only regression probe for same-World Model recreation; no source edits."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
normal=V/'native-scoped-catalog-normal/validation-20261018-132210-585372';q=read(normal/'Debug-qualified-20261018-133514-354151/qualified.json');assert q['passed']==156
inputs=read(normal/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native33199-ordinary-actor-ABA'/stamp;out.mkdir(parents=True)
runtime=P/'target/current/native33199-ordinary-actor-ABA'/stamp;runtime.mkdir(parents=True)
original=P/'tests/native_world_lighting_scene.rs';body=original.read_bytes();(out/'original-tests.rs').write_bytes(body)
test=r'''

#[test]
fn ordinary_actor_cue_never_replays_after_same_world_model_recreation() {
 let _guard=source_test_guard();let(mut world,env)=current();let own=world.own_actor().unwrap().id;
 let mut packet=vec![0x20];packet.extend(own.to_le_bytes());packet.push(68);assert!(world.accept(&packet).unwrap());
 let sequence=world.actors[&own].action_sequence;let authority=serde_json::to_value(&world).unwrap();let mut view=Presentation::default();
 let mut observations=Vec::new();let mut held=None;let mut old_results=Vec::new();let mut sink=LegacySink::default();
 for phase in 0..2 {
  let deadline=Instant::now()+Duration::from_secs(90);
  loop {
   view.tick(root(),2,&world,&env,true);assert!(view.error.is_none(),"{:?}",view.error);
   if let Some(frame)=view.frame.as_mut(){
    if let Some(cue)=frame.legacy_audio.iter().find(|c|c.event().is_some_and(|e|e.actor==own&&e.sequence==sequence&&e.action==68)) {
     observations.push(serde_json::json!({"phase":phase,"event":cue.event(),"sound":cue.sound().sound_id(),"raw_sounds":frame.sounds.len(),"cues":frame.legacy_audio.len()}));
     if phase==0{held=Some(cue.clone());} break;
    }
    frame.sounds.clear();frame.legacy_audio.clear();
   }
   assert!(Instant::now()<deadline,"actor event wait phase {phase}");std::thread::sleep(Duration::from_millis(10));
  }
  if phase==0 {
   view.cancel();let scope=view.audio_scope(2,world.generation);let cue=held.as_ref().unwrap();
   let first=cue.submit(&scope,&mut sink).unwrap();let repeat=cue.clone().submit(&scope,&mut sink).unwrap();
   assert_eq!(first,lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Stale);assert_eq!(repeat,lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::AlreadyConsumed);assert!(sink.plays.is_empty());
   old_results.push(format!("{first:?}"));old_results.push(format!("{repeat:?}"));
   while !view.quiescent(){view.tick(root(),2,&world,&env,false);assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
  }
 }
 assert_eq!(serde_json::to_value(&world).unwrap(),authority);
 let o=serde_json::json!({"same_authority_world_unchanged":true,"observations":observations,"old_results":old_results,"old_deliveries":sink.plays.len(),"own":own,"sequence":sequence,"live_server_sessions":0,"physical_audio_verified":false,"normal_product_source_modified":false});
 std::fs::write(std::env::var_os("LITO_AUDIO_ABA_OBSERVATION").unwrap(),serde_json::to_vec_pretty(&o).unwrap()).unwrap();
 assert!(observations.len()<2,"ordinary actor cue replayed after same-World Model recreation");
}

'''
(out/'tests.rs').write_bytes(body+b'\n'+test.encode('utf-8'));(out/'runner.py').write_bytes(Path(__file__).read_bytes())
libraries={};native=set()
for line in (normal/'Debug/test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for path in row['filenames']:
            if path.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/' in path.replace('\\','/'):libraries[name]=path
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';exe=runtime/'ordinary-audio-ABA.exe'
args=[str(rust),str(out/'tests.rs'),'--test','--crate-name','ordinary_audio_ABA','--edition=2021','--target','x86_64-pc-windows-msvc','-C','opt-level=0','-C','debuginfo=1','-C','debug-assertions=yes','-C','overflow-checks=yes','-o',str(exe)]
for n,p in libraries.items():args+=['--extern',n+'='+p]
for path in [P/'target/current/x86_64-pc-windows-msvc/debug/deps',P/'target/current/debug/deps']:args+=['-L','dependency='+str(path)]
for path in sorted(native):args+=['-L',path]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='running_resource_only_same_World_Model_recreation_probe',source_manifest_sha256=q['source_manifest_sha256'],qualified_Debug_product_sha256=q['normal']['sha256'],source_test_sha256=sha(original),probe_sha256=sha(out/'tests.rs'),normal_library=dict(path=libraries['lito_modern'],sha256=sha(libraries['lito_modern'])),libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,normal_product_source_modified=False,live_server_sessions=0,resource_only_wait_bound_seconds=90,acceptance_8s_changed=False,gameplay_qualification_granted=False,G11_qualified=False)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),LITO_AUDIO_ABA_OBSERVATION=str(out/'observation.json'),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('ABA_PROBE='+str(out),flush=True)
try:
    for label,command in [('compile',args),('probe',[str(exe),'--exact','ordinary_actor_cue_never_replays_after_same_world_model_recreation','--nocapture','--test-threads=1'])]:
        with(out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
            child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=command);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=180)
            except subprocess.TimeoutExpired:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];child.terminate();job['exit_code']=child.wait();raise
            state['owned_process']=None;save(out/'receipt.json',state)
            if label=='compile':assert job['exit_code']==0
            else:
                assert job['exit_code']==101 and 'ordinary actor cue replayed after same-World Model recreation' in (out/'probe.stderr.txt').read_text('utf-8-sig')
    observation=read(out/'observation.json');assert observation['same_authority_world_unchanged'] and observation['old_deliveries']==0 and len(observation['observations'])==2
    for n,h in inputs.items():assert sha(ROOT/n)==h
    state.update(status='reproduced_same_World_new_Model_ordinary_actor_audio_replay_failure',product=str(exe),product_sha256=sha(exe),observation_sha256=sha(out/'observation.json'),baseline_child_exit_code=101,baseline_regression_passed=False)
except BaseException as error:state.update(status='failed_probe_driver_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(probe=str(out),baseline_child_exit_code=101,regression_passed=False,ordinary_replay_reproduced=True,normal_product_source_modified=False),ensure_ascii=False))
