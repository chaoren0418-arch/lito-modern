"""Export 20,879 pinned bytes for an independent decoder; no gameplay or model."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
broad=ROOT/'驗證紀錄/current/native-default111/20261017-135059-659904'
c=read(broad/'cohort.json');done=read(broad/'completion-observation.json')
assert c['status']=='independently_qualified_current_default111_both_profiles' and done['observed_terminal_exit_code']==0
assert done['recorded_original_owned_processes_all_terminal']
debug=c['profiles']['Debug'];q=read(debug['qualification']);assert sha(debug['qualification'])==debug['qualification_sha256']
assert q['actual_test_executables']==112 and q['counts']['failed']==0
debug_run=Path(debug['run']);pins=read(debug_run/'compiled-inputs.json');assert len(pins)==1029
libraries={};native=set()
for line in (debug_run/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='compiler-artifact' and not row['profile']['test'] and row['target']['name']=='lito_modern':
        paths=[p for p in row['filenames'] if p.endswith('.rlib')];assert len(paths)==1
        libraries['lito_modern']=dict(path=paths[0],sha256=sha(paths[0]))
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
assert set(libraries)=={'lito_modern'}
inventory=ROOT/'驗證紀錄/current/native-attack-five-resource-inspection/20261017-144953-005315/asset-inventory.json'
expected={m['name']:m for m in read(inventory)['members'] if m['name'] in ['1815-0.spr','1816-0.spr','1818-0.spr']};assert len(expected)==3
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-attack-five-resource-inspection'/('blank-export-'+stamp);out.mkdir()
runtime=PROJECT/'target/current/native-attack-five-resource-inspection'/('blank-export-'+stamp);runtime.mkdir()
samples=runtime/'samples';samples.mkdir()
source=ROOT/'維護工具/candidates/native-attack-five/export_blanks.rs'
(out/'probe.rs').write_bytes(source.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
state=dict(status='prepared_three_pinned_raw_samples',output=str(out),samples=str(samples),source_manifest_sha256=c['source_manifest_sha256'],libraries=libraries,
    inventory=str(inventory),inventory_sha256=sha(inventory),source_sha256=sha(source),owned_process=None,process_history=[],new_sessions=0,new_casts=0,
    sprite_decode_executed=False,product_source_modified=False,GUI_verified=False,whole_goal_complete=False)
env=dict(os.environ,TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
def check():
    assert sha(source)==state['source_sha256'] and sha(inventory)==state['inventory_sha256']
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for v in libraries.values():assert sha(v['path'])==v['sha256']
def execute(command,label):
    check()
    with (out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
        p=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        kernel=ctypes.WinDLL('kernel32',use_last_error=True);times=[wintypes.FILETIME() for _ in range(4)]
        kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
        assert kernel.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
        ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        item=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=command)
        state['owned_process']=item;state['process_history'].append(item);save(out/'receipt.json',state)
        item['exit_code']=p.wait();state['owned_process']=None;save(out/'receipt.json',state)
        assert item['exit_code']==0,label+' failed; preserve before repair'
    check()
print('BLANK_SAMPLE_EXPORT='+str(out),flush=True)
try:
    exe=runtime/'export-blank-samples.exe'
    command=[str(Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'),str(out/'probe.rs'),'--edition=2021','--crate-name','lito_blank_samples','--target','x86_64-pc-windows-msvc','-C','opt-level=0','-C','debuginfo=1',
        '-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'--extern','lito_modern='+libraries['lito_modern']['path'],'-o',str(exe)]
    for n in sorted(native):command+=['-L',n]
    execute(command,'rustc');state['product_sha256']=sha(exe)
    execute([str(exe),str(samples)],'export')
    rows=[]
    for name,m in expected.items():
        p=samples/name;assert sha(p)==m['raw_sha256'] and p.stat().st_size==m['raw_bytes']
        rows.append(dict(name=name,path=str(p),raw_sha256=sha(p),raw_bytes=p.stat().st_size))
    assert sum(r['raw_bytes'] for r in rows)==20879
    state.update(status='exported_three_exact_pinned_raw_samples_no_decoder',members=rows)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(status=state['status'],output=str(out),samples=str(samples),bytes=20879),ensure_ascii=False),flush=True)
