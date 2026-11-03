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
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native33199-admission-map-generation'/stamp;out.mkdir(parents=True)
runtime=P/'target/current/native33199-admission-map-generation'/stamp;runtime.mkdir(parents=True)
original=P/'tests/native_world_lighting_scene.rs';body=original.read_bytes();(out/'original-tests.rs').write_bytes(body)
test=r'''

#[test]
fn candidate_direct_model_map_change_cannot_keep_previous_world_death() {
 let _guard=source_test_guard();let(mut a,_)=current();let own=a.own_actor().unwrap().id;
 let packet=|action:u8|{let mut p=vec![0x20];p.extend(own.to_le_bytes());p.push(action);p};
 assert!(a.accept(&packet(8)).unwrap());let stop=AtomicBool::new(false);let mut model=Model::load(root(),&a,&stop).unwrap();model.draw_at(&a,&stop,0).unwrap();
 let old=model.draw_at(&a,&stop,60_000_000_000).unwrap();let old_actor=old.receipt["actors"].as_array().unwrap().iter().find(|r|r["id"]==own).unwrap();assert_eq!(old_actor["phase"],"death");assert_eq!(old_actor["action_complete"],true);
 let(mut b,_)=current();assert!(b.accept(&packet(68)).unwrap());assert_eq!(a.actors[&own].incarnation,b.actors[&own].incarnation);assert_eq!(a.actors[&own].action_sequence,b.actors[&own].action_sequence);
 b.generation=a.generation+1;b.map_id=Some(500);{let actor=b.actors.get_mut(&own).unwrap();actor.x=32768;actor.y=32832;}
 let authority=serde_json::to_value(&b).unwrap();let frame=model.draw_at(&b,&stop,61_000_000_000).unwrap();assert_eq!(serde_json::to_value(&b).unwrap(),authority);let actor=frame.receipt["actors"].as_array().unwrap().iter().find(|r|r["id"]==own).unwrap();
 let o=serde_json::json!({"authority_unchanged":true,"actor_alive":!b.actors[&own].dead,"old_world_generation":a.generation,"new_world_generation":b.generation,"old_map":a.map_id,"new_map":b.map_id,"new_actor_phase":actor["phase"],"new_actor_action":actor["action"],"new_actor_receipt":actor,"synthetic_authority_resource_fixture":true,"live_server_sessions":0,"physical_audio_verified":false});
 std::fs::write(std::env::var_os("LITO_AUDIO_ABA_OBSERVATION").unwrap(),serde_json::to_vec_pretty(&o).unwrap()).unwrap();assert_ne!(actor["phase"],"death","previous world death leaked into new alive actor after map change");
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
candidate_receipt=read(V/'native33199-ordinary-admission-v2-ready/20261018-145825-685763/receipt.json');libraries['lito_modern']=candidate_receipt['diagnostic_library'];assert sha(libraries['lito_modern'])==candidate_receipt['diagnostic_library_sha256']
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
    for label,command in [('compile',args),('probe',[str(exe),'--exact','candidate_direct_model_map_change_cannot_keep_previous_world_death','--nocapture','--test-threads=1'])]:
        with(out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
            child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=command);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=180)
            except subprocess.TimeoutExpired:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];child.terminate();job['exit_code']=child.wait();raise
            state['owned_process']=None;save(out/'receipt.json',state)
            if label=='compile':assert job['exit_code']==0
            else:
                assert job['exit_code']==101 and 'previous world death leaked into new alive actor after map change' in (out/'probe.stderr.txt').read_text('utf-8-sig')
    observation=read(out/'observation.json');assert observation['authority_unchanged'] and observation['actor_alive'] and observation['new_actor_phase']=='death'
    for n,h in inputs.items():assert sha(ROOT/n)==h
    state.update(status='reproduced_isolated_candidate_v2_cross_world_death_pose_leak',product=str(exe),product_sha256=sha(exe),observation_sha256=sha(out/'observation.json'),baseline_child_exit_code=101,baseline_regression_passed=False)
except BaseException as error:state.update(status='failed_probe_driver_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(probe=str(out),baseline_child_exit_code=101,regression_passed=False,candidate_cross_world_death_leak_reproduced=True,normal_product_source_modified=False),ensure_ascii=False))
