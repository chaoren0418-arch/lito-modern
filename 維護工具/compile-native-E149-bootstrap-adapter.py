"""javac only for the staged original-resource adapter; never load classes."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
rd=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
save=lambda p,v:Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('candidate',type=Path);a=ap.parse_args()
cp=a.candidate.resolve(strict=True);assert cp.is_relative_to(R/'維護工具/candidates')
c=rd(cp);w=rd(R/'工作佇列.json')
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user']
assert c['formal_target_sha256']==w['current_formal_reference_identity']['formal_sha256']
q=rd(w['current_development_typecheck_qualification']['qualification']);inputs=rd(Path(q['run'])/'base-compiled-inputs.json')
source=Path(c['source']);jar=Path(c['formal_reference']);javac=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javac.exe')
assert sha(source)==c['source_sha256'] and sha(c['static_manifest'])==c['static_manifest_sha256'] and sha(c['SQL'])==c['SQL_sha256']
pins={str(p):sha(p) for p in [cp,source,jar,javac,Path(c['static_manifest']),Path(c['SQL'])]}
def guard():
    assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
    assert all(sha(p)==h for p,h in pins.items()) and sha(w['current_formal_reference_identity']['formal'])==c['formal_target_sha256']
    assert all(sha(p)==h for p,h in q['normal_product_pins'].items())
guard();out=V/'native-E149-bootstrap-adapter-compile'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'classes').mkdir();(out/'scratch').mkdir();(out/'compiler.py').write_bytes(Path(__file__).read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts])
    ticks=ts[0].dwLowDateTime|ts[0].dwHighDateTime<<32
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
command=[str(javac),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(out/'scratch'),'-encoding','UTF-8','-proc:none','--release','17','-classpath',str(jar),'-d',str(out/'classes'),str(source)]
record=dict(status='compiling_adapter_only',candidate=str(cp),candidate_sha256=sha(cp),formal_target_sha256=c['formal_target_sha256'],
    source_manifest_sha256=c['source_manifest_sha256'],input_pins=pins,normal_product_pins=q['normal_product_pins'],base_compiled_inputs=str(Path(q['run'])/'base-compiled-inputs.json'),
    pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),command=command,
    tests_executed=0,SQL_executed=False,database_started=False,original_factories_invoked=False,classes_instantiated=0,desktop_inputs=0,normal_sources_modified=False)
save(out/'cohort.json',record);print(json.dumps(dict(run=str(out),scope='javac only, no Java class/SQL execution'),ensure_ascii=False),flush=True)
with(out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
    child=subprocess.Popen(command,cwd=out,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    record['compiler']=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))));save(out/'cohort.json',record)
    try:child.wait(timeout=120)
    except BaseException:
        if child.poll() is None:
            assert birth(wintypes.HANDLE(int(child._handle)))==record['compiler']['creation_utc'];child.kill();child.wait();record['exact_owned_cleanup']=True
        raise
    finally:
        record.update(compiler_exit_code=child.returncode,finished_utc=datetime.now(timezone.utc).isoformat(),status='adapter_compilation_only_complete' if child.returncode==0 else 'compile_failure_preserved')
        record['classes']={str(p):sha(p) for p in(out/'classes').rglob('*.class')};record['logs']={str(out/n):sha(out/n) for n in ['stdout.log','stderr.log']}
        save(out/'cohort.json',record);guard()
assert child.returncode==0 and (out/'classes/local/lito381/server/NativeRecoveryProductionBootstrap.class').is_file()
print(json.dumps(dict(run=str(out),compiler_exit_code=0,classes=len(record['classes']),tests_executed=0,SQL_executed=False),ensure_ascii=False))
