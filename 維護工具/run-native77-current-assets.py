"""Actual explicit namespace load using a preserved current native library."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
prepared=ROOT/'維護工具/candidates/native77-current-assets';p=read(prepared/'preparation.json')
assert sha(p['source'])==p['source_sha256'] and sha(p['candidate'])==p['candidate_sha256']
normal=ROOT/'驗證紀錄/current/native544-integration/validation-20261017-172422-276459'
n=read(normal/'cohort.json');done=read(normal/'completion-observation.json');assert done['observed_terminal_exit_code']==0 and done['recorded_owned_all_terminal']
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1029
old=ROOT/'驗證紀錄/current/default-native-regression/20261017-135100-713-Debug/build/stdout.log'
libraries={};native=set()
for line in old.read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='compiler-artifact' and not row['profile']['test'] and row['target']['name'] in {'lito_modern','anyhow','serde_json','sha2'}:
        paths=[x for x in row['filenames'] if x.endswith('.rlib')]
        if paths:assert len(paths)==1;libraries[row['target']['name']]=dict(path=paths[0],sha256=sha(paths[0]))
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
assert set(libraries)=={'lito_modern','anyhow','serde_json','sha2'}
start=next(j for j in n['process_history'] if j['label']=='Debug/normal-build')['creation_utc']
end=next(j for j in n['process_history'] if j['label']=='Debug/version')['creation_utc']
mtime=datetime.fromtimestamp(Path(libraries['lito_modern']['path']).stat().st_mtime,tz=timezone.utc)
assert datetime.fromisoformat(start)<=mtime<=datetime.fromisoformat(end),'Native library must be emitted inside the owned current normal build'
asset=Path(r'D:\lineage開發\0-客戶端');files=[asset/'launcher/TW13081901.pak',asset/'Sound/717.wav']+[asset/f'Sprite{i:02}.idx' for i in range(3,7)]
original={str(f):sha(f) for f in files};metadata={str(asset/f'Sprite{i:02}.pak'):dict(bytes=(asset/f'Sprite{i:02}.pak').stat().st_size,mtime_ns=(asset/f'Sprite{i:02}.pak').stat().st_mtime_ns) for i in range(3,7)}
assert not (asset/'TW13081901.pak').exists()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native77-current-assets'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native77-current-assets'/stamp;runtime.mkdir(parents=True)
body=(prepared/'probe_body.rs').read_text(encoding='utf-8-sig');wrapper=out/'probe.rs';wrapper.write_text(body.replace('__ASSETS_PATH__',Path(p['candidate']).as_posix()),encoding='utf-8',newline='\n')
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'candidate-assets.rs').write_bytes(Path(p['candidate']).read_bytes());save(out/'preparation.json',p)
state=dict(status='prepared_current_TW_asset_candidate',output=str(out),source_manifest_sha256=n['source_manifest_sha256'],source_files=1028,compiled_input_files=1029,
    normal_qualification=str(normal/'completion-observation.json'),normal_qualification_sha256=sha(normal/'completion-observation.json'),current_native_library_emission_window=[start,end],
    libraries=libraries,normal_mains=done['profiles'],preparation=p,original_input_pins=original,original_PAK_metadata=metadata,
    owned_process=None,process_history=[],production_source_modified=False,new_sessions=0,new_casts=0,playback_verified=False,audio_device_verified=False,GUI_verified=False,whole_goal_complete=False)
env={k:v for k,v in os.environ.items() if not k.startswith('LITO_')};env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),LITO_ASSET_PROBE_OUTPUT=str(out/'asset-report.json'))
def check():
    for name,h in inputs.items():assert sha(ROOT/name)==h,name
    for v in libraries.values():assert sha(v['path'])==v['sha256']
    for name,h in original.items():assert sha(name)==h,name
    for name,m in metadata.items():assert (Path(name).stat().st_size,Path(name).stat().st_mtime_ns)==(m['bytes'],m['mtime_ns'])
    for profile,v in n['profiles'].items():assert sha(v['normal']['path'])==v['normal']['sha256']
def execute(command,label):
    check();d=out/label;d.mkdir()
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        times=[wintypes.FILETIME() for _ in range(4)];kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
        assert kernel.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in times])
        ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32);job=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=command,label=label)
        state['owned_process']=job;state['process_history'].append(job);save(out/'receipt.json',state)
        job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state)
    check();assert job['exit_code']==0,label+' failed; preserve before repair'
print('CURRENT_TW_ASSET_CANDIDATE='+str(out),flush=True)
try:
    exe=runtime/'current-tw-assets-tests.exe'
    cmd=[str(Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'),str(wrapper),'--edition=2021','--crate-name','lito_current_tw_assets','--test','--target','x86_64-pc-windows-msvc','-C','opt-level=0','-C','debuginfo=1',
        '-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
    for name,v in libraries.items():cmd+=['--extern',name+'='+v['path']]
    for value in sorted(native):cmd+=['-L',value]
    execute(cmd,'compile');state['product']=dict(path=str(exe),sha256=sha(exe));save(out/'receipt.json',state)
    execute([str(exe),'--list','--format','terse'],'inventory');execute([str(exe),'--list','--ignored','--format','terse'],'ignored-inventory')
    execute([str(exe),'--test-threads=1'],'unit');execute([str(exe),'current_tw_assets_match_existing_independent_goldens','--exact','--ignored','--test-threads=1'],'actual-assets')
    report=read(out/'asset-report.json');assert report['frames']==72 and report['empty_frames']==10 and report['encoded_bytes']==716815
    assert bytes(report['sound_file_sha256']).hex().upper()==original[str(asset/'Sound/717.wav')]
    state.update(status='executed_current_TW_asset_entry_candidate',asset_report_sha256=sha(out/'asset-report.json'))
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],frames=72,encoded_bytes=716815),ensure_ascii=False),flush=True)
