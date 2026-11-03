"""Build current Debug library and rerun exactly the two preserved failures."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
SOURCE='4018AE86474EA2977F0083034E81A86D4F77F1C51EDF828B2456B369876DA24D'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('application',type=Path);a=ap.parse_args();app=a.application.resolve(strict=True)
assert app.is_relative_to(V/'native-installed-resource-profile-normal')
application=read(app/'application.json');assert application['source_manifest_sha256']==SOURCE and application['test_only_source_modified']
candidate=read(application['candidate']);assert candidate['test_only_fixture_repair']and candidate['production_prefix_unchanged']
names=sorted(candidate['exact_regression_tests']);assert len(names)==2
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==SOURCE and not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
inputs=read(app/'applied-compiled-inputs.json');assert len(inputs)==1049
assert not {n:v for n,v in os.environ.items()if v and(n.startswith('CARGO_PROFILE_')or n in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(h):
 t=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(x)for x in t])
 ticks=t[0].dwLowDateTime|(t[0].dwHighDateTime<<32)
 return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
out=V/'native-render-test-fixture-regression'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'compiled-inputs.json').write_bytes((app/'applied-compiled-inputs.json').read_bytes());(out/'source-inputs.json').write_bytes((app/'applied-source-inputs.json').read_bytes())
state=dict(status='prepared_focused_Debug_library_fixture_regression',source_manifest_sha256=SOURCE,application=str(app),
    pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,expected_tests=names,
    full_library_verified=False,normal_products_qualified=False,actual_network_sessions=0,GUI_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
env={n:v for n,v in os.environ.items()if not n.startswith('LITO_')};env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),RUSTC=str(rust/'rustc.exe'),RUSTDOC=str(rust/'rustdoc.exe'));env['PATH']=str(rust)+os.pathsep+env['PATH']
def guard():
 for n,h in inputs.items():assert sha(R/n)==h,n
 assert sha(out/'runner.py')==sha(Path(__file__))
def execute(command,label):
 guard();d=out/label;d.mkdir()
 with(d/'stdout.log').open('wb')as stdout,(d/'stderr.log').open('wb')as stderr:
  child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
  row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=label,exit_code=None)
  state['process_history'].append(row);state['owned_process']=row;save(out/'cohort.json',state)
  try:code=child.wait()
  except BaseException:
   if child.poll()is None:
    assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc'];child.terminate();child.wait(timeout=3)
   raise
  finally:row['exit_code']=child.returncode;state['owned_process']=None;save(out/'cohort.json',state)
 guard();return code
try:
 state['status']='compiling_focused_Debug_library_fixture_regression';save(out/'cohort.json',state)
 command=[str(rust/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--lib']
 assert execute(command,'build')==0
 messages=[json.loads(l)for l in(out/'build/stdout.log').read_text('utf-8-sig').splitlines()if l.startswith('{')]
 artifacts=[m for m in messages if m.get('reason')=='compiler-artifact'and m.get('executable')and m['profile']['test']]
 assert len(artifacts)==1 and artifacts[0]['target']['name']=='lito_modern'and artifacts[0]['features']==[]
 exe=Path(artifacts[0]['executable']).resolve(strict=True);assert exe.is_relative_to(P/'target/current')
 state.update(product=str(exe),product_sha256=sha(exe),status='running_focused_Debug_library_fixture_regression');save(out/'cohort.json',state)
 with zipfile.ZipFile(out/'product.zip','x',zipfile.ZIP_DEFLATED)as z:z.write(exe,exe.name)
 code=execute([str(exe),*names,'--exact','--test-threads=1'],'tests')
 text=(out/'tests/stdout.log').read_text('utf-8-sig')
 found=re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$',text,re.M)
 assert len(found)==2 and {n for n,_ in found}==set(names)
 totals=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
 assert len(totals)==1
 state.update(actual_Rust_exit_code=code,counts=dict(passed=int(totals[0][1]),failed=int(totals[0][2]),ignored=int(totals[0][3]),filtered_out=int(totals[0][5])));save(out/'cohort.json',state)
 assert code==0 and totals[0]==('ok','2','0','0','0','2456')and all(result=='ok'for _,result in found)
 state['status']='executed_two_focused_Debug_library_fixture_regressions'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),counts=state['counts'],full_library_verified=False),ensure_ascii=False),flush=True)
