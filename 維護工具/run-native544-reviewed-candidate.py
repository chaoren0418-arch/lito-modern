"""Same real-asset test is red on the baseline and required green on candidate."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
prepared=ROOT/'維護工具/candidates/native544-reviewed';p=read(prepared/'preparation.json')
assert sha(p['source'])==p['source_sha256'] and sha(p['candidate'])==p['candidate_sha256']
assert sha(p['reference'])==p['reference_sha256'];reference=Path(p['reference']);rq=read(reference)
assert rq['reviewed_null_slots']==32 and rq['nonempty_reference_frames']==31
report=reference.parent.parent/'report.json';assert sha(report)==rq['raw_files']['report.json']
broad=ROOT/'驗證紀錄/current/native-default111/20261017-135059-659904';c=read(broad/'cohort.json')
assert c['status']=='independently_qualified_current_default111_both_profiles'
debug=Path(c['profiles']['Debug']['run']);pins=read(debug/'compiled-inputs.json');assert len(pins)==1029
libraries={};native=set()
for line in (debug/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='compiler-artifact' and not row['profile']['test'] and row['target']['name'] in {'lito_modern','serde_json','sha2'}:
        paths=[x for x in row['filenames'] if x.endswith('.rlib')]
        if paths:assert len(paths)==1;libraries[row['target']['name']]=dict(path=paths[0],sha256=sha(paths[0]))
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
assert set(libraries)=={'lito_modern','serde_json','sha2'}
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native544-reviewed-candidate'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native544-reviewed-candidate'/stamp;runtime.mkdir(parents=True)
body=prepared/'probe_body.rs';(out/'probe_body.rs').write_bytes(body.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
for name,src in [('baseline-sprite.rs',Path(p['source'])),('candidate-sprite.rs',Path(p['candidate']))]:(out/name).write_bytes(src.read_bytes())
save(out/'preparation.json',p);save(out/'compiled-inputs.json',pins)
state=dict(status='prepared_isolated_exact_marker_comparison',output=str(out),runtime=str(runtime),source_manifest_sha256=c['source_manifest_sha256'],preparation=p,libraries=libraries,
    independent_reference=str(reference),independent_reference_sha256=sha(reference),report=str(report),report_sha256=sha(report),body_sha256=sha(body),
    process_history=[],owned_process=None,products={},production_source_modified=False,normal_product_built=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
env={k:v for k,v in os.environ.items() if not k.startswith('LITO_')}
env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),LITO_CSHARP_REPORT=str(report),LITO_CSHARP_REPORT_SHA256=sha(report))
def check():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    assert sha(p['source'])==p['source_sha256'] and sha(p['candidate'])==p['candidate_sha256'] and sha(body)==state['body_sha256']
    assert sha(reference)==state['independent_reference_sha256'] and sha(report)==state['report_sha256']
    for v in libraries.values():assert sha(v['path'])==v['sha256']
def execute(command,label,expected=0):
    check();directory=out/label;directory.mkdir()
    with (directory/'stdout.log').open('wb') as stdout,(directory/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
        assert kernel.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in times])
        ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        job=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=command,label=label,expected_exit_code=expected)
        state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
        job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state)
    check();assert job['exit_code']==expected,label+' failed; preserve original before repair'
print('EXACT_MARKER_CANDIDATE='+str(out),flush=True)
try:
    template=body.read_text(encoding='utf-8-sig')
    for label,source in [('baseline',out/'baseline-sprite.rs'),('candidate',out/'candidate-sprite.rs')]:
        wrapper=out/(label+'-probe.rs');wrapper.write_text(template.replace('__SPRITE_PATH__',source.as_posix()),encoding='utf-8',newline='\n')
        exe=runtime/(label+'-tests.exe')
        command=[str(Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'),str(wrapper),'--edition=2021','--crate-name','lito_reviewed544_'+label,'--test','--target','x86_64-pc-windows-msvc','-C','opt-level=0','-C','debuginfo=1',
            '-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
        for name,info in libraries.items():command+=['--extern',name+'='+info['path']]
        for n in sorted(native):command+=['-L',n]
        state['status']='compiling_isolated_'+label;execute(command,label+'-compile')
        state['products'][label]=dict(path=str(exe),sha256=sha(exe),wrapper=str(wrapper),wrapper_sha256=sha(wrapper));save(out/'receipt.json',state)
        execute([str(exe),'--list','--format','terse'],label+'-inventory')
        execute([str(exe),'--list','--ignored','--format','terse'],label+'-ignored-inventory')
        env['LITO_SAMPLE_COMPARISON_OUTPUT']=str(out/(label+'-comparison.json'))
        if label=='candidate':execute([str(exe),'--test-threads=1'],'candidate-unit')
        execute([str(exe),'exact_owned_samples_match_independent_csharp','--exact','--ignored','--nocapture','--test-threads=1'],label+'-samples',101 if label=='baseline' else 0)
        data=read(out/(label+'-comparison.json'))
        if label=='baseline':assert data['accepted_files']==0 and len(data['errors'])==3
        else:assert (data['accepted_files'],data['frames'],data['empty_frames'],data['flat_pixel_matches'],data['compact_pixel_matches'])==(3,63,32,31,31) and not data['errors']
        print(json.dumps(dict(label=label,comparison=data),ensure_ascii=False),flush=True)
    state['status']='executed_isolated_baseline_red_candidate_green_exact_marker'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
