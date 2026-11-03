"""Isolated catalog lifetime/cancellation/root tests against candidate bodies."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current';T=ROOT/'維護工具'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
candidate=T/'candidates/native8CC5-scoped-catalog';prep=read(candidate/'preparation.json')
for pin in [*prep['original_files'].values(),*prep['candidate_files'].values()]:assert sha(pin['path'])==pin['sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native8CC5-scoped-catalog'/('component-'+stamp);out.mkdir(parents=True)
runtime=P/'target/current/native8CC5-scoped-catalog'/('component-'+stamp);runtime.mkdir(parents=True)
tests='''
    #[test]
    fn model_slot_reads_and_parses_once_then_releases_pinned_files() {
        let fixture=Fixture::valid();let cancel=AtomicBool::new(false);let mut slot=None;test_counts::reset();
        let a=SharedActionCatalog::reuse_for_model(&fixture.root,&cancel,&mut slot).unwrap().catalog() as *const _;
        for _ in 0..8 {let b=SharedActionCatalog::reuse_for_model(&fixture.root,&cancel,&mut slot).unwrap();assert_eq!(a,b.catalog() as *const _);assert_eq!(b.sha256().as_slice(),hex(SHA));}
        assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:1,parses:1});
        #[cfg(windows)] for name in ["Text.idx","Text.pak"] {assert!(fs::OpenOptions::new().write(true).open(fixture.root.join(name)).is_err());}
        drop(slot);
        for name in ["Text.idx","Text.pak"] {fs::OpenOptions::new().write(true).open(fixture.root.join(name)).unwrap();}
    }
    #[test]
    fn model_slot_cached_cancellation_precedes_missing_root_access() {
        let fixture=Fixture::valid();let mut slot=None;let cancel=AtomicBool::new(false);test_counts::reset();
        SharedActionCatalog::reuse_for_model(&fixture.root,&cancel,&mut slot).unwrap();
        assert!(matches!(SharedActionCatalog::reuse_for_model(Path::new("missing-cancelled-root"),&AtomicBool::new(true),&mut slot),Err(SharedCatalogError::Cancelled)));
        assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:1,parses:1});
        assert_eq!(slot.as_ref().unwrap().sha256().as_slice(),hex(SHA));
    }
    #[test]
    fn model_slot_refuses_cross_root_reuse_and_retains_original_owner() {
        let a=Fixture::valid();let b=Fixture::valid();let cancel=AtomicBool::new(false);let mut slot=None;test_counts::reset();
        SharedActionCatalog::reuse_for_model(&a.root,&cancel,&mut slot).unwrap();
        assert!(matches!(SharedActionCatalog::reuse_for_model(&b.root,&cancel,&mut slot),Err(SharedCatalogError::Resource(ResourceError::UnsafePath(_)))));
        assert_eq!(slot.as_ref().unwrap().root(),a.root);assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:1,parses:1});
        drop(slot);let mut next=None;SharedActionCatalog::reuse_for_model(&b.root,&cancel,&mut next).unwrap();
        assert_eq!(next.as_ref().unwrap().root(),b.root);assert_eq!(test_counts::snapshot(),Counts{load_attempts:2,reads:2,parses:2});
    }
    #[test]
    fn model_slot_ambiguous_failure_never_publishes_a_cached_context() {
        let fixture=Fixture::new(&hex(DUPLICATE_INDEX),&hex(PAK));let mut slot=None;test_counts::reset();
        assert!(matches!(SharedActionCatalog::reuse_for_model(&fixture.root,&AtomicBool::new(false),&mut slot),Err(SharedCatalogError::Resource(ResourceError::AmbiguousEntry{count:2,..}))));
        assert!(slot.is_none());assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:0,parses:0});
    }
'''
body=(candidate/'shared_action_catalog.rs').read_text('utf-8-sig');where=body.rfind('\n}');assert where>0
(out/'shared_action_catalog.tests.rs').write_text(body[:where]+tests+body[where:],encoding='utf-8',newline='\n')
modules={'resource_cipher':P/'src/modern_lito/resource_cipher.rs','resources':P/'src/modern_lito/resources.rs','sprite_actions':P/'src/modern_lito/sprite_actions.rs','shared_action_catalog':out/'shared_action_catalog.tests.rs'}
wrapper='pub mod modern_lito {\n'+''.join('#[path = '+json.dumps(path.as_posix(),ensure_ascii=False)+']\npub mod '+name+';\n' for name,path in modules.items())+'}\n'
(out/'component.rs').write_text(wrapper,encoding='utf-8',newline='\n');(out/'runner.py').write_bytes(Path(__file__).read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='running',runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),owned_process=None,process_history=[],candidate_preparation_sha256=sha(candidate/'preparation.json'),component_source_sha256=sha(out/'component.rs'),test_source_sha256=sha(out/'shared_action_catalog.tests.rs'),module_sources={name:dict(path=str(path),sha256=sha(path)) for name,path in modules.items()},profiles={},normal_product_source_modified=False,gameplay_qualification_granted=False)
def execute(args,label,env):
    with (out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
        child=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=args);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
        try:job['exit_code']=child.wait(timeout=180)
        except subprocess.TimeoutExpired:
            assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];child.terminate();job['exit_code']=child.wait();raise
        state['owned_process']=None;save(out/'receipt.json',state);assert job['exit_code']==0,(label,job['exit_code'])
env=dict(os.environ,CARGO_MANIFEST_DIR=str(runtime),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
normal=V/'native-legacy-audio-normal/validation-20261018-104256-823814'
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
print('COMPONENT='+str(out),flush=True)
try:
    for profile in ['Debug','Release']:
        libraries={}
        for line in (normal/profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
            if not line.startswith('{'):continue
            row=json.loads(line)
            if row.get('reason')=='compiler-artifact':
                for path in row['filenames']:
                    if path.endswith('.rlib'):
                        name=row['target']['name'].replace('-','_')
                        if name not in libraries or '/x86_64-pc-windows-msvc/' in path.replace('\\','/'):libraries[name]=path
        exe=runtime/('catalog-'+profile+'.exe');args=[str(rust),str(out/'component.rs'),'--test','--crate-name','scoped_catalog_component','--edition=2021','--target','x86_64-pc-windows-msvc','--extern','sha2='+libraries['sha2'],'-L','dependency='+str(P/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'deps'),'-o',str(exe)]
        args+=['-C','opt-level=0','-C','debuginfo=1','-C','debug-assertions=yes','-C','overflow-checks=yes'] if profile=='Debug' else ['-C','opt-level=z','-C','lto=thin','-C','codegen-units=16','-C','strip=symbols','-C','debug-assertions=no','-C','overflow-checks=no']
        execute(args,profile+'-compile',env);state['profiles'][profile]=dict(product=str(exe),sha256=sha(exe),sha2=dict(path=libraries['sha2'],sha256=sha(libraries['sha2'])))
        execute([str(exe),'modern_lito::shared_action_catalog::tests::','--test-threads=1','--nocapture'],profile+'-tests',env)
        log=(out/(profile+'-tests.stdout.txt')).read_text('utf-8-sig');assert '11 passed; 0 failed; 0 ignored' in log
    for pin in prep['original_files'].values():assert sha(pin['path'])==pin['sha256']
    state['status']='both_profiles_catalog_component_11_passed_each'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(component=str(out),profiles=2,passed_each=11,normal_product_source_modified=False),ensure_ascii=False))
