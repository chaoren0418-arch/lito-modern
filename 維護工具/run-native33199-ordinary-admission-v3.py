"""Compile five isolated bodies and test retired, fresh and new-connection S4A cues."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current';T=ROOT/'維護工具'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
normal=V/'native-scoped-catalog-normal/validation-20261018-132210-585372';nc=read(normal/'cohort.json');no=read(normal/'completion-observation.json');assert no['observed_terminal_exit_code']==0 and no['recorded_original_owned_processes_all_terminal']
inputs=read(normal/'compiled-inputs.json');candidate=T/'candidates/native33199-ordinary-admission-v3';prep=read(candidate/'preparation.json');assert prep['source_manifest_sha256']==nc['source_manifest_sha256']
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads'] and queue['source_and_corresponding_QA_frozen']
def guard():
    for n,h in inputs.items():assert sha(ROOT/n)==h
    for key in ['original_files','candidate_files']:
        for pin in prep[key].values():assert sha(pin['path'])==pin['sha256']
guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native33199-ordinary-admission-v3'/stamp;out.mkdir(parents=True);runtime=P/'target/current/native33199-ordinary-admission-v3'/stamp;runtime.mkdir(parents=True)
for n,pin in prep['candidate_files'].items():(out/n).write_bytes(Path(pin['path']).read_bytes())
baseline=V/'native33199-ordinary-audio-ABA/20261018-135628-012455';body=(baseline/'tests.rs').read_text('utf-8-sig')
needle='    assert!(frame.legacy_audio.is_empty()&&frame.sounds.is_empty(),"ordinary wire cue replayed after same-World Model recreation");';assert body.count(needle)==1
body=body.replace(needle,needle+'''
    assert!(world.accept(&[0x4a,0,109,0]).unwrap());let fresh_authority=serde_json::to_value(&world).unwrap();
    wire_admission_consume(&mut view,2,&world,&env,2,&mut sink);assert_eq!(sink.plays.len(),1);assert_eq!(serde_json::to_value(&world).unwrap(),fresh_authority);
    view.cancel();while !view.quiescent(){view.tick(root(),2,&world,&env,false);std::thread::sleep(Duration::from_millis(10));}
    let(mut new_world,_)=current();assert!(new_world.accept(&[0x4a,0,109,0]).unwrap());
    let mut new_env=State::default();new_env.begin(3).unwrap();new_env.map_changed(3,new_world.map_id.unwrap(),Some(0)).unwrap();
    let new_authority=serde_json::to_value(&new_world).unwrap();wire_admission_consume(&mut view,3,&new_world,&new_env,1,&mut sink);
    assert_eq!(sink.plays.len(),2);assert_eq!(serde_json::to_value(&new_world).unwrap(),new_authority);
''',1)
body+='''
fn wire_admission_consume(view:&mut Presentation,connection:u64,world:&World,env:&State,sequence:u64,sink:&mut LegacySink) {
    let deadline=Instant::now()+Duration::from_secs(90);
    let cue=loop {
        view.tick(root(),connection,world,env,true);assert!(view.error.is_none(),"{:?}",view.error);
        if let Some(frame)=view.frame.as_ref(){if let Some(cue)=frame.legacy_audio.iter().find(|c|c.event().is_some_and(|e|e.actor==0&&e.sequence==sequence)){assert_eq!(frame.legacy_audio.len(),1);assert_eq!(frame.sounds.len(),1);break cue.clone();}}
        assert!(Instant::now()<deadline,"fresh wire cue timed out");std::thread::sleep(Duration::from_millis(10));
    };
    let scope=view.audio_scope(connection,world.generation);
    assert!(matches!(cue.submit(&scope,sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Played(_)));
    assert_eq!(cue.clone().submit(&scope,sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::AlreadyConsumed);
}
#[test]
fn ordinary_pending_wire_is_retired_before_worker_can_publish() {
    let _source_guard=source_test_guard();let(mut world,env)=current();assert!(world.accept(&[0x4a,0,109,0]).unwrap());
    let authority=serde_json::to_value(&world).unwrap();let mut view=Presentation::default();let deadline=Instant::now()+Duration::from_secs(90);
    view.tick(root(),2,&world,&env,true);assert!(!view.quiescent());view.cancel();
    while !view.quiescent(){view.tick(root(),2,&world,&env,false);assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
    loop {view.tick(root(),2,&world,&env,true);assert!(view.error.is_none(),"{:?}",view.error);if let Some(frame)=view.frame.as_ref(){assert!(frame.sounds.is_empty()&&frame.legacy_audio.is_empty(),"cancelled submitted wire event reappeared");break;}assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
    assert_eq!(serde_json::to_value(&world).unwrap(),authority);assert!(world.accept(&[0x4a,0,109,0]).unwrap());
    let fresh_authority=serde_json::to_value(&world).unwrap();let mut sink=LegacySink::default();wire_admission_consume(&mut view,2,&world,&env,2,&mut sink);assert_eq!(sink.plays.len(),1);assert_eq!(serde_json::to_value(&world).unwrap(),fresh_authority);
}
'''
body+='\n'+(candidate/'actor-regressions.rs').read_text('utf-8-sig')
(out/'actor-regressions.rs').write_bytes((candidate/'actor-regressions.rs').read_bytes())
(out/'tests.rs').write_text(body,encoding='utf-8',newline='\n');(out/'runner.py').write_bytes(Path(__file__).read_bytes())
lines=[];previous=False;overrides=0
for line in (P/'src/modern_lito/mod.rs').read_text('utf-8-sig').splitlines():
    path=re.match(r'\s*#\[path\s*=\s*"([^"\n]+)"\]\s*$',line)
    if path:lines.append('#[path = '+json.dumps((P/'src/modern_lito'/path[1]).resolve(strict=True).as_posix(),ensure_ascii=False)+']');previous=True;continue
    decl=re.match(r'^(?:(?:pub(?:\([^)]*\))?)\s+)?mod\s+(\w+);\s*$',line)
    if decl:
        if not previous:
            name=decl[1]+'.rs';path=out/name if name in prep['candidate_files'] else P/'src/modern_lito'/name
            if name in prep['candidate_files']:overrides+=1
            if not path.exists():path=P/'src/modern_lito'/decl[1]/'mod.rs'
            assert path.is_file();lines.append('#[path = '+json.dumps(path.as_posix(),ensure_ascii=False)+']')
        previous=False
    lines.append(line)
assert overrides==5;(out/'resolved_mod.rs').write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')
wrapper='#[path = '+json.dumps((out/'resolved_mod.rs').as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n#[path = '+json.dumps((P/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\n';(out/'lib.rs').write_text(wrapper,encoding='utf-8',newline='\n')
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
libraries.pop('lito_modern');rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';rlib=runtime/'liblito_modern_wire_admission.rlib';exe=runtime/'wire-admission.exe'
common=['--edition=2021','--target','x86_64-pc-windows-msvc','-C','opt-level=0','-C','debuginfo=1','-C','debug-assertions=yes','-C','overflow-checks=yes']
for n,p in libraries.items():common+=['--extern',n+'='+p]
for path in [P/'target/current/x86_64-pc-windows-msvc/debug/deps',P/'target/current/debug/deps']:common+=['-L','dependency='+str(path)]
for path in sorted(native):common+=['-L',path]
commands=[('library',[str(rust),str(out/'lib.rs'),'--crate-name','lito_modern','--crate-type','rlib','-C','metadata=wire_admission_33199','-o',str(rlib)]+common),('compile',[str(rust),str(out/'tests.rs'),'--test','--crate-name','wire_admission_regressions','--extern','lito_modern='+str(rlib),'-o',str(exe)]+common),('regressions',[str(exe),'ordinary_','--nocapture','--test-threads=1'])]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='running_isolated_S4A_admission_candidate',source_manifest_sha256=nc['source_manifest_sha256'],candidate_preparation_sha256=sha(candidate/'preparation.json'),source_overlays=prep['candidate_files'],probe_sha256=sha(out/'tests.rs'),libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},diagnostic_library=str(rlib),product=str(exe),runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),owned_process=None,process_history=[],normal_product_source_modified=False,diagnostic_request_API_extension=False,live_server_sessions=0,physical_audio_verified=False,whole_goal_complete=False)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],LITO_AUDIO_ABA_OBSERVATION=str(out/'observation.json'),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('WIRE_ADMISSION_CANDIDATE='+str(out),flush=True)
try:
    for label,args in commands:
        guard()
        with(out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
            child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=args);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=180)
            except subprocess.TimeoutExpired:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];child.terminate();job['exit_code']=child.wait();raise
            state['owned_process']=None;save(out/'receipt.json',state);assert job['exit_code']==0,(label,job['exit_code'])
    text=(out/'regressions.stdout.txt').read_text('utf-8-sig');assert '5 passed; 0 failed; 0 ignored' in text
    o=read(out/'observation.json');assert o['new_Model_legacy_cues']==o['new_Model_raw_sounds']==0 and o['same_authority_world_unchanged']
    guard();state.update(status='isolated_S4A_model_recreation_pending_retirement_and_fresh_connection_regressions_passed',diagnostic_library_sha256=sha(rlib),product_sha256=sha(exe),passed=5,failed=0,ignored=0,ordinary_actor_published_and_pending_same_world_verified=True,ordinary_actor_source_ABA_qualified=False)
except BaseException as error:state.update(status='failed_candidate_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(candidate_run=str(out),passed=5,failed=0,normal_product_source_modified=False,ordinary_actor_published_and_pending_same_world_verified=True,ordinary_actor_source_ABA_qualified=False),ensure_ascii=False))
