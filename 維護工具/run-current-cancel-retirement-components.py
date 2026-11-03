"""Original rental-return assertions plus cancel regression, before product apply."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
source=w['current_product_identity']['source_manifest_sha256'];proposal=R/'維護工具/candidates/native-cancel-retirement-20261020-113749-366354/candidate.json';c=read(proposal)
assert c['base_source_manifest_sha256']==source and not c['source_applied']
counter=w['current_native_cancel_retirement_counterexample'];assert counter['qualified_counterexample'] and sha(counter['qualification'])==counter['qualification_sha256']
normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run']);inputs=read(normal/'compiled-inputs.json')
out=V/'native-cancel-retirement-components'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'runner.py').write_bytes(Path(__file__).read_bytes());pins={}
test=Path(c['candidate_files']['LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs']);body=test.read_text('utf-8-sig');test_names=re.findall(r'(?m)^#\[test\]\nfn (\w+)\(',body);assert len(test_names)==len(set(test_names)) and 'cancel_live_world_retires_snapshot_without_losing_generation' in test_names
haste=read(c['related_haste_candidate']);modules={}
for name in ['native_ground_pickup','native_item_actions','native_portal','native_room']:modules[name]=P/'src/modern_lito'/(name+'.rs')
for name in ['native_skills','native_skill_selection']:modules[name]=Path(haste['candidate_files']['LiTo381-Modern/src/modern_lito/'+name+'.rs'])
for mode,origin in [('baseline',P/'src/modern_lito/native_login.rs'),('candidate',Path(c['candidate_files']['LiTo381-Modern/src/modern_lito/native_login.rs']))]:
    text=origin.read_text('utf-8-sig').partition('/// Exercises real authentication using isolated test credentials supplied via stdin.\npub fn probe(\n')[0]
    replacements={name:P/'src/modern_lito'/name for name in ['native_battle_withdrawal_tests.rs','native_return_server_negative_tests.rs','native_teleport_controller_tests.rs']};replacements['native_rental_return_tests.rs']=test
    for name,path in replacements.items():
        before='#[path = "'+name+'"]';assert text.count(before)==1
        text=text.replace(before,'#[path = r#"'+str(path)+'"#]');pins[str(path)]=sha(path)
    target=out/(mode+'-controller.rs');target.write_text(text,encoding='utf-8',newline='\n');pins[str(target)]=sha(target)
    wrapper='mod candidate {\n pub use lito_modern::modern_lito::*;\n'
    for name,path in {'native_login':target,**modules}.items():wrapper+=' #[path = r#"'+str(path)+'"#]\n pub mod '+name+';\n';pins[str(path)]=sha(path)
    wrapper+='}\nmod modern_lito {pub use crate::candidate::*;}\n';path=out/(mode+'.rs');path.write_text(wrapper,encoding='utf-8',newline='\n');pins[str(path)]=sha(path)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(x) for x in ts]);ticks=ts[0].dwLowDateTime|ts[0].dwHighDateTime<<32
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='preparing_cancel_components',source_manifest_sha256=source,candidate=str(proposal),candidate_sha256=sha(proposal),compiled_inputs=inputs,generated_and_referenced_pins=pins,pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,profiles={},test_names=test_names,normal_product_modified=False,candidate_applied=False,GUI_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
def guard():
    assert all(sha(R/n)==h for n,h in inputs.items()) and all(sha(p)==h for p,h in pins.items())
    assert all(sha(R/n)==h for n,h in c['before'].items()) and all(sha(p)==c['after'][n] for n,p in c['candidate_files'].items())
env={n:v for n,v in os.environ.items() if not n.startswith('LITO_')};env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),CARGO_MANIFEST_DIR=str(P))
def execute(command,folder,expected):
    guard();folder.mkdir(parents=True)
    with(folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=folder,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,exit_code=None);state['process_history'].append(row);state['owned_process']=row;save(out/'cohort.json',state)
        try:child.wait(timeout=300)
        except subprocess.TimeoutExpired:child.kill();child.wait(timeout=10);row['exact_owned_timeout_cleanup']=True;raise
        finally:row.update(exit_code=child.returncode,finished_utc=datetime.now(timezone.utc).isoformat());state['owned_process']=None;save(out/'cohort.json',state)
    assert child.returncode==expected,(str(folder),child.returncode,expected);guard()
rustc=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rustc.exe');target='x86_64-pc-windows-msvc'
try:
    for profile in ['Debug','Release']:
        broad=w['current_native_catalog_broad_qualification']['profiles'][profile];build=Path(broad['run'])/'build/stdout.log';messages=[json.loads(l) for l in build.read_text('utf-8-sig').splitlines() if l.startswith('{')];deps=P/'target/current'/target/profile.lower()/'deps';extern={}
        for name in ['lito_modern','anyhow','serde','serde_json','png','zeroize','windows','sha2']:
            paths={str(Path(f).resolve(strict=True)) for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']==name for f in m['filenames'] if f.endswith('.rlib') and Path(f).parent==deps};assert len(paths)==1,(profile,name,paths);extern[name]=paths.pop()
        state['profiles'][profile]=dict(extern={n:dict(path=p,sha256=sha(p)) for n,p in extern.items()},modes={})
        for mode in ['baseline','candidate']:
            work=out/profile/mode;work.mkdir(parents=True);exe=work/'cancel-components.exe';command=[str(rustc),'--test',str(out/(mode+'.rs')),'--edition=2021','--target',target,'--crate-name','cancel_'+profile.lower()+'_'+mode,'-o',str(exe),'-L','dependency='+str(deps),'-L','dependency='+str(P/'target/current'/profile.lower()/'deps')]
            command+=['-C','opt-level=0','-C','debuginfo=2'] if profile=='Debug' else ['-C','opt-level=z','-C','lto=thin','-C','codegen-units=16','-C','strip=symbols']
            for n,p in extern.items():command+=['--extern',n+'='+p]
            for p in sorted({p for m in messages if m.get('reason')=='build-script-executed' for p in m['linked_paths']}):command+=['-L',p]
            state.update(status='compiling_cancel_components',active_profile=profile,active_mode=mode);execute(command,work/'compile',0)
            execute([str(exe),'candidate::native_login::rental_return_tests::','--test-threads=1'],work/'run',101 if mode=='baseline' else 0)
            results=dict(re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$',(work/'run/stdout.log').read_text('utf-8-sig'),re.M));expected={'candidate::native_login::rental_return_tests::'+n:('FAILED' if mode=='baseline' and n=='cancel_live_world_retires_snapshot_without_losing_generation' else 'ok') for n in test_names};assert results==expected
            state['profiles'][profile]['modes'][mode]=dict(executable=str(exe),executable_sha256=sha(exe),results=results);save(out/'cohort.json',state);print(json.dumps(dict(profile=profile,mode=mode,tests=len(results),expected_counterexample=mode=='baseline')),flush=True)
    state.update(status='executed_cancel_retirement_components_both_profiles')
except BaseException as e:state.update(status='failed_preserved',error=repr(e));raise
finally:guard();state['finished_utc']=datetime.now(timezone.utc).isoformat();save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
