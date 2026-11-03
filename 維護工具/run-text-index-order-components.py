"""Compare the unchanged failing test with the exact reference-only candidate."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,re
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
save=lambda p,x:Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('candidate',type=Path);a=ap.parse_args();cp=a.candidate.resolve(strict=True);c=read(cp)
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'] and c['before_source_manifest_sha256']==w['current_product_identity']['source_manifest_sha256']
assert c['only_cfg_test_reference_changed'] and not c['Runtime_changed'] and sha(c['comparison'])==c['comparison_sha256']
normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run']);inputs=read(normal/'compiled-inputs.json')
assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
name=next(iter(c['candidate_files']));original=R/name;candidate=Path(c['candidate_files'][name]);cipher=P/'src/modern_lito/resource_cipher.rs'
assert sha(original)==c['before_files'][name] and sha(candidate)==c['after_files'][name]
out=V/'native-text-index-order-components'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'scratch').mkdir()
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|ts[0].dwHighDateTime<<32
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='prepared',source_manifest_sha256=c['before_source_manifest_sha256'],candidate=str(cp),candidate_sha256=sha(cp),pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,profiles={},Runtime_changed=False,product_source_modified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state);(out/'runner.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
env={n:v for n,v in os.environ.items() if not n.startswith('LITO_')};env.update(TEMP=str(out/'scratch'),TMP=str(out/'scratch'),LITO_SHARED_RESOURCE_ROOT=r'D:\lineage開發\0-客戶端')
pins={str(original):sha(original),str(candidate):sha(candidate),str(cipher):sha(cipher),str(cp):sha(cp)}
def guard():
    assert all(sha(R/n)==h for n,h in inputs.items()) and all(sha(p)==h for p,h in pins.items())
def run(command,folder,expected):
    guard();folder.mkdir();stdout=(folder/'stdout.log').open('wb');stderr=(folder/'stderr.log').open('wb');child=subprocess.Popen(command,cwd=folder,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,output=str(folder),exit_code=None);state['process_history'].append(row);state['owned_process']=row;save(out/'cohort.json',state)
    try:code=child.wait(timeout=180)
    except BaseException:
        if child.poll() is None:assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc'];child.kill();child.wait();row['owned_timeout_cleanup']=True
        raise
    finally:row.update(exit_code=child.returncode,finished_utc=datetime.now(timezone.utc).isoformat());stdout.close();stderr.close();state['owned_process']=None;save(out/'cohort.json',state)
    guard();assert code==expected,(command,code,expected);return row
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';target='x86_64-pc-windows-msvc';test='resources::tests::real_shared_text_matches_csharp_reference'
try:
    for profile in ['Debug','Release']:
        build=normal/profile/'test-build/stdout.log';messages=[json.loads(l) for l in build.read_text('utf-8-sig').splitlines() if l.startswith('{')];deps=P/'target/current'/target/profile.lower()/'deps'
        artifacts={str(Path(f).resolve(strict=True)) for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']=='sha2' for f in m['filenames'] if f.endswith('.rlib') and Path(f).parent==deps};assert len(artifacts)==1
        sha2=artifacts.pop();pins[sha2]=sha(sha2);state['profiles'][profile]={};work=out/profile;work.mkdir()
        for label,source in [('baseline',original),('candidate',candidate)]:
            wrapper=work/(label+'.rs');wrapper.write_text('#[path = r#"'+str(cipher)+'"#]\nmod resource_cipher;\n#[path = r#"'+str(source)+'"#]\nmod resources;\n',encoding='utf-8');exe=work/(label+'.exe')
            command=[str(rust),str(wrapper),'--edition=2021','--test','--target',target,'--crate-name','text_order_'+label+'_'+profile.lower(),'-o',str(exe),'-L','dependency='+str(deps),'--extern','sha2='+sha2]
            command+=['-C','opt-level=0','-C','debuginfo=2'] if profile=='Debug' else ['-C','opt-level=z','-C','lto=thin','-C','codegen-units=16','-C','strip=symbols']
            state.update(status='compiling',active_profile=profile,active_variant=label);save(out/'cohort.json',state);run(command,work/(label+'-build'),0)
            state.update(status='testing');save(out/'cohort.json',state);row=run([str(exe),test,'--exact','--ignored','--test-threads=1'],work/(label+'-text'),101 if label=='baseline' else 0)
            text=(Path(row['output'])/'stdout.log').read_text('utf-8-sig');assert re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',text,re.M)==[(test,'FAILED' if label=='baseline' else'ok')]
            if label=='baseline':assert 'left: 37497' in text and 'right: 37496' in text
            info=dict(executable=str(exe),executable_sha256=sha(exe),text_result='expected_original_failure' if label=='baseline' else'passed',original_test_body_preserved=label=='baseline')
            if label=='candidate':
                row=run([str(exe),'--test-threads=1'],work/'candidate-default',0);text=(Path(row['output'])/'stdout.log').read_text('utf-8-sig');counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text);assert len(counts)==1 and counts[0][1]=='0' and counts[0][3:]==('0','0');info['default_counts']=dict(zip(['passed','failed','ignored','measured','filtered'],map(int,counts[0])))
            state['profiles'][profile][label]=info;save(out/'cohort.json',state)
    state['status']='executed_baseline_counterexample_and_candidate_both_profiles'
except BaseException as e:state.update(status='failed_preserved',error=repr(e));raise
finally:state.update(pinned_inputs=pins,finished_utc=datetime.now(timezone.utc).isoformat());guard();save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],profiles=state['profiles']),ensure_ascii=False))
