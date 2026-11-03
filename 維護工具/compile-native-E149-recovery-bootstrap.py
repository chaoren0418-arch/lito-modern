"""Compile integrated fixture Java plus two Rust metadata targets; no tests."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,tomllib
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('candidate',type=Path);ap.add_argument('--java-only',action='store_true');a=ap.parse_args()
cp=a.candidate.resolve(strict=True);assert cp.is_relative_to(R/'維護工具/candidates');c=read(cp)
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
assert c['formal_target_sha256']==w['current_formal_reference_identity']['formal_sha256']
qpath=Path(w['current_development_typecheck_qualification']['qualification']);q=read(qpath)
assert sha(qpath)==w['current_development_typecheck_qualification']['qualification_sha256']
prior=Path(q['run']);cohort=read(prior/'cohort.json');inputs=read(prior/'base-compiled-inputs.json')
assert all(q['candidates'][k]['sha256']==v['sha256'] for k,v in w['current_isolated_development']['candidates'].items())
sources=[cp.parent/n for n in c['candidate_files']];java=[p for p in sources if p.suffix=='.java'];rust=[p for p in sources if p.suffix=='.rs']
assert len(java)==3 and len(rust)==1 and all(sha(cp.parent/n)==h for n,h in c['candidate_files'].items())
jar=Path(c['formal_reference']);peer=P/'tests/oracle/NativePeerHarness.java'
javac=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javac.exe')
pins={str(p):sha(p) for p in [cp,*sources,Path(c['contract']),jar,peer,javac,qpath,prior/'cohort.json',prior/'base-compiled-inputs.json']}
for profile in ['Debug','Release']:
    pins[q['profiles'][profile]['library']['metadata']]=q['profiles'][profile]['library']['metadata_sha256']
    for v in cohort['profiles'][profile]['extern'].values():pins[v['path']]=v['sha256']
def guard():
    assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
    assert all(sha(p)==h for p,h in pins.items())
    assert sha(w['current_formal_reference_identity']['formal'])==sha(jar)==c['formal_target_sha256']
    assert all(sha(p)==h for p,h in q['normal_product_pins'].items())
guard();out=V/'native-E149-recovery-bootstrap-compile'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'classes').mkdir();(out/'scratch').mkdir();(out/'compiler.py').write_bytes(Path(__file__).read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    times=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in times])
    ticks=times[0].dwLowDateTime|times[0].dwHighDateTime<<32
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
record=dict(status='compiling_integrated_fixture_only',candidate=str(cp),candidate_sha256=sha(cp),input_pins=pins,
    compilation_scope='Java-only; unchanged Rust qualifications retained' if a.java_only else 'Java plus two Rust formal metadata targets',
    source_manifest_sha256=c['base_source_manifest_sha256'],formal_target_sha256=c['formal_target_sha256'],
    pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],
    base_compiled_inputs=str(prior/'base-compiled-inputs.json'),normal_product_pins=q['normal_product_pins'],
    reused_library_metadata_qualification=str(qpath),reused_library_metadata_qualification_sha256=sha(qpath),
    tests_executed=0,classes_instantiated=0,SQL_executed=False,database_started=False,original_factories_invoked=False,
    desktop_inputs=0,normal_sources_modified=False,existing_thirteen_file_overlay_unchanged=True)
save=lambda:(out/'cohort.json').write_bytes(data(record))
save();print(json.dumps(dict(run=str(out),scope=record['compilation_scope']),ensure_ascii=False),flush=True)
env={n:v for n,v in os.environ.items() if not n.startswith('LITO_')}
env.update(TEMP=str(out/'scratch'),TMP=str(out/'scratch'),CARGO_MANIFEST_DIR=str(P),
    CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'])
def compile_one(name,command):
    guard();folder=out/name;folder.mkdir()
    with(folder/'stdout.log').open('xb') as stdout,(folder/'stderr.log').open('xb') as stderr:
        child=subprocess.Popen(command,cwd=folder,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        row=dict(name=name,pid=child.pid,creation_utc=birth(int(child._handle)),command=command)
        record['process_history'].append(row);save()
        try:child.wait(timeout=120)
        except BaseException:
            if child.poll() is None:child.terminate();child.wait(timeout=15)
            raise
        finally:
            row.update(exit_code=child.poll(),finished_utc=datetime.now(timezone.utc).isoformat(),
                logs={str(folder/n):sha(folder/n) for n in ['stdout.log','stderr.log']});save();guard()
    assert child.returncode==0,(name,folder)
try:
    compile_one('Java',[str(javac),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(out/'scratch'),'-encoding','UTF-8',
        '-proc:none','--release','17','-classpath',str(jar),'-d',str(out/'classes'),str(peer),*[str(p) for p in java]])
    for profile in ([] if a.java_only else ['Debug','Release']):
        expected=prior/profile/'teleport-recovery-formal-code-compile'
        rows=[r for r in cohort['process_history'] if Path(r['output'])==expected];assert len(rows)==1
        command=list(rows[0]['command']);pins[command[0]]=sha(command[0])
        original=str(prior/'generated/LiTo381-Modern/tests/native_teleport_failure.rs');assert command.count(original)==1
        command[command.index(original)]=str(rust[0]);command[command.index('-o')+1]=str(out/(profile+'.rmeta'))
        assert '--emit=metadata' in command and '--test' in command
        compile_one(profile,command)
    record.update(status='integrated_fixture_compilation_only_complete',classes={str(p):sha(p) for p in(out/'classes').rglob('*.class')},
        metadata={str(out/(p+'.rmeta')):dict(sha256=sha(out/(p+'.rmeta')),bytes=(out/(p+'.rmeta')).stat().st_size) for p in ([] if a.java_only else ['Debug','Release'])})
except BaseException:
    record['status']='compile_failure_preserved';raise
finally:
    record['finished_utc']=datetime.now(timezone.utc).isoformat();save();guard()
assert len(record['classes'])==6
print(json.dumps(dict(run=str(out),classes=len(record['classes']),compiler_exit_codes=[r['exit_code'] for r in record['process_history']],tests_executed=0,SQL_executed=False),ensure_ascii=False))
