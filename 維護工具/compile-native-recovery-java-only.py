"""Compile formal fixture source only. Never launch Java classes or fixtures."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
rd=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
save=lambda p,v:Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
w=rd(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
cp=Path(w['current_native_cancel_recovery_candidate']['candidate']);c=rd(cp)
assert sha(cp)==w['current_native_cancel_recovery_candidate']['candidate_sha256']
source=c['base_source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']
qa=rd(w['current_development_typecheck_qualification']['qualification'])
inputs=rd(Path(qa['run'])/'base-compiled-inputs.json')
java=cp.parent/'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'
peer=P/'tests/oracle/NativePeerHarness.java'
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'
expected=c.get('formal_target_sha256','75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371')
if c.get('formal_reference'):
    jar=Path(c['formal_reference'])
    assert sha(w['current_formal_reference_identity']['formal'])==expected
java_sources=[cp.parent/n for n in c['candidate_files'] if n.endswith('.java')]
javac=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javac.exe')
assert sha(java)==c['candidate_files']['LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java']
assert sha(jar)==expected
assert java in java_sources and all(sha(p)==c['candidate_files'][p.relative_to(cp.parent).as_posix()] for p in java_sources)
pins={str(p):sha(p) for p in [cp,*java_sources,peer,jar,javac]}
def guard():
    assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
    assert all(sha(p)==h for p,h in pins.items())
    assert all(sha(p)==h for p,h in qa['normal_product_pins'].items())
guard();out=V/'native-recovery-java-compile'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'classes').mkdir();(out/'scratch').mkdir();(out/'compiler.py').write_bytes(Path(__file__).read_bytes())
kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetCurrentProcess.restype=wintypes.HANDLE
kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert kernel.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts])
    ticks=ts[0].dwLowDateTime | ts[0].dwHighDateTime<<32
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
command=[str(javac),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(out/'scratch'),'-encoding','UTF-8','-proc:none','--release','17','-classpath',str(jar),'-d',str(out/'classes'),str(peer),*[str(p) for p in java_sources]]
record=dict(status='compiling_fixture_only',candidate=str(cp),candidate_sha256=sha(cp),source_manifest_sha256=source,
    pipeline=dict(pid=os.getpid(),creation_utc=birth(kernel.GetCurrentProcess())),command=command,input_pins=pins,formal_reference=str(jar),formal_reference_sha256=expected,
    tests_executed=0,game_or_fixture_executions=0,desktop_inputs=0,normal_source_modified=False,normal_product_pins=qa['normal_product_pins'])
save(out/'cohort.json',record);print(json.dumps(dict(run=str(out),scope='javac only; no java invocation'),ensure_ascii=False),flush=True)
with(out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    child=subprocess.Popen(command,cwd=out,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    record['compiler']=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))))
    save(out/'cohort.json',record)
    try:child.wait(timeout=120)
    except BaseException:
        if child.poll() is None:
            assert birth(wintypes.HANDLE(int(child._handle)))==record['compiler']['creation_utc']
            child.kill();child.wait();record['exact_owned_cleanup']=True
        raise
    finally:
        record.update(compiler_exit_code=child.returncode,finished_utc=datetime.now(timezone.utc).isoformat(),status='javac_only_completed' if child.returncode==0 else 'compile_failure_preserved')
        record['classes']={str(p):sha(p) for p in (out/'classes').rglob('*.class')}
        record['logs']={str(out/n):sha(out/n) for n in ['stdout.log','stderr.log']}
        save(out/'cohort.json',record);guard()
assert child.returncode==0 and record['classes']
assert (out/'classes/local/lito381/server/NativeTeleportFailureHarness.class').is_file()
print(json.dumps(dict(run=str(out),compiler_exit_code=0,classes=len(record['classes']),tests_executed=0),ensure_ascii=False))
