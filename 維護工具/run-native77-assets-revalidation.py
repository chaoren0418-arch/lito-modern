"""Revalidate current archive inputs with the existing unchanged native images."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern';ASSETS=Path(r'D:\lineage開發\0-客戶端')
OLD=ROOT/'驗證紀錄/current/native77-scene-integration/validation-20261017-183727-278843'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
c=read(OLD/'cohort.json');assert c['status']=='executed_current_native77_normals_and_related_scopes' and len(c['process_history'])==96 and all(j['exit_code']==0 for j in c['process_history'])
assert read(OLD/'terminal-only-observation.json')['recorded_owned_all_terminal']
inputs=read(OLD/'compiled-inputs.json');assert len(inputs)==1031
for n,h in inputs.items():assert sha(ROOT/n)==h
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native77-assets-revalidation'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native77-assets-revalidation'/stamp;runtime.mkdir(parents=True);(out/'runner.py').write_bytes(Path(__file__).read_bytes())
state=dict(status='pinning_current_readonly_archives',output=str(out),source_manifest_sha256=c['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,
    original_cohort=str(OLD),original_cohort_sha256=sha(OLD/'cohort.json'),original_terminal_tool_chunk='c82688',original_resource_change=str(OLD/'Release-resource-readback-failure.json'),
    profiles={},owned_process=None,process_history=[],normal_mains={p:v['normal'] for p,v in c['profiles'].items()},production_source_modified=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;kernel.GetCurrentProcess.restype=wintypes.HANDLE
def created(handle):
    times=[wintypes.FILETIME() for _ in range(4)];assert kernel.GetProcessTimes(handle,*[ctypes.byref(t) for t in times]);ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'original-runner-process.json',dict(pid=os.getpid(),creation_utc=created(kernel.GetCurrentProcess()),parent_pid=os.getppid()))
save(out/'receipt.json',state);print('NATIVE77_ASSET_REVALIDATION='+str(out),flush=True)
archive_files=sorted({*ASSETS.glob('*.idx'),*ASSETS.glob('*.pak'),ASSETS/'launcher/TW13081901.pak',ASSETS/'Sound/717.wav'})
def metadata(f):s=f.stat();return dict(bytes=s.st_size,mtime_ns=s.st_mtime_ns)
def sound_metadata():return {str(f):metadata(f) for f in sorted((ASSETS/'Sound').glob('*.wav'))}
initial_meta={str(f):metadata(f) for f in archive_files};initial_sounds=sound_metadata();asset_pins={}
for f in archive_files:
    asset_pins[str(f)]=dict(**initial_meta[str(f)],sha256=sha(f));assert metadata(f)==initial_meta[str(f)]
    state['pinned_archive_count']=len(asset_pins);save(out/'receipt.json',state)
assert all(metadata(Path(n))==m for n,m in initial_meta.items()) and sound_metadata()==initial_sounds
save(out/'asset-inputs-before.json',dict(archives=asset_pins,sound_metadata=initial_sounds,scope='all flat PAK/IDX archives, explicit launcher TW and Sound717 bytes; Sound directory metadata'))
state.update(status='current_archives_pinned',archive_inputs_sha256=sha(out/'asset-inputs-before.json'));save(out/'receipt.json',state)
locator=ROOT/'驗證紀錄/current/default-native-regression/20261017-135100-713-Debug/build/stdout.log';libraries={};native=set()
for line in locator.read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='compiler-artifact' and not row['profile']['test'] and row['target']['name'] in {'lito_modern','anyhow','serde_json','sha2','serde'}:
        paths=[x for x in row['filenames'] if x.endswith('.rlib')]
        if paths:assert len(paths)==1;libraries[row['target']['name']]=dict(path=paths[0],sha256=sha(paths[0]))
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
assert set(libraries)=={'lito_modern','anyhow','serde_json','sha2','serde'};state['libraries']=libraries
first=read(ROOT/'驗證紀錄/current/native77-scene-integration/validation-20261017-183221-124368/cohort.json')
lo=next(j['creation_utc'] for j in first['process_history'] if j['label']=='Debug/normal-build');hi=next(j['creation_utc'] for j in first['process_history'] if j['label']=='Debug/version')
assert datetime.fromisoformat(lo)<=datetime.fromtimestamp(Path(libraries['lito_modern']['path']).stat().st_mtime,tz=timezone.utc)<=datetime.fromisoformat(hi)
body=(ROOT/'維護工具/candidates/native77-current-assets/probe_body.rs').read_text(encoding='utf-8-sig');line='    #[path="__ASSETS_PATH__"] pub mod disintegrate_assets;\n';assert body.count(line)==1
body=body.replace(line,'');assert '__ASSETS_PATH__' not in body;wrapper=out/'current-assets-probe.rs';wrapper.write_text(body,encoding='utf-8',newline='\n')
env={k:v for k,v in os.environ.items() if not k.startswith('LITO_')};env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),LITO_ASSET_PROBE_OUTPUT=str(out/'current-asset-report.json'))
def check():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,m in initial_meta.items():assert metadata(Path(n))==m,n
    assert sound_metadata()==initial_sounds
    for p,info in c['profiles'].items():
        for kind in ['normal','test']:assert sha(info[kind]['path'])==info[kind]['sha256']
        for info in info['integration_targets'].values():assert sha(info['path'])==info['sha256']
def execute(command,label):
    check();d=out/label;d.mkdir(parents=True)
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=created(wintypes.HANDLE(int(child._handle))),command=command,label=label);state['owned_process']=job;state['process_history'].append(job);save(out/'receipt.json',state)
        job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state)
    check();assert job['exit_code']==0,label+' failed; preserve before repair'
    return d
def counts(d):
    rows=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',(d/'stdout.log').read_text(encoding='utf-8-sig'));assert len(rows)==1 and rows[0][0]=='ok'
    return dict(zip(['passed','failed','ignored','measured','filtered_out'],map(int,rows[0][1:])))
try:
    exe=runtime/'current-assets-goldens.exe';rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
    command=[str(rust),str(wrapper),'--edition=2021','--crate-name','native77_current_assets_revalidation','--test','--target','x86_64-pc-windows-msvc','-C','opt-level=0','-C','debuginfo=1','-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
    for n,v in libraries.items():command+=['--extern',n+'='+v['path']]
    for value in sorted(native):command+=['-L',value]
    execute(command,'golden-compile');state['golden_image']=dict(path=str(exe),sha256=sha(exe));save(out/'receipt.json',state)
    execute([str(exe),'--list','--format','terse'],'golden-inventory');execute([str(exe),'--list','--ignored','--format','terse'],'golden-ignored-inventory')
    assert counts(execute([str(exe),'current_tw_assets_match_existing_independent_goldens','--exact','--ignored','--test-threads=1'],'golden-current-resources'))==dict(passed=1,failed=0,ignored=0,measured=0,filtered_out=0)
    report=read(out/'current-asset-report.json');assert report['frames']==72 and report['empty_frames']==10 and report['encoded_bytes']==716815
    state['current_selected_SPRs_equal_independent_old_goldens']=True;save(out/'receipt.json',state)
    print('CURRENT_SELECTED_4_SPR_72_FRAMES_MATCH_INDEPENDENT_GOLDENS',flush=True)
    for profile,info in c['profiles'].items():
        state.update(status='executing_current_archive_revalidation',active_profile=profile);state['profiles'][profile]=dict(normal=info['normal'],test=info['test'],integration_targets={});save(out/'receipt.json',state)
        result=counts(execute([info['test']['path'],'modern_lito::native_disintegrate_tests::native77_probe::actual_','--ignored','--test-threads=1'],profile+'/native77-actual-assets'));assert result==dict(passed=10,failed=0,ignored=0,measured=0,filtered_out=2416)
        state['profiles'][profile]['actual_assets']=result
        for target,item in info['integration_targets'].items():
            d=execute([item['path'],'--test-threads=1'],profile+'/'+target);result=counts(d);assert result==item['counts']
            rows=dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',(d/'stdout.log').read_text(encoding='utf-8-sig'),re.M));assert set(rows)==set(item['registered_tests']) and {n for n,v in rows.items() if v=='ignored'}==set(item['registered_ignored'])
            state['profiles'][profile]['integration_targets'][target]=dict(path=item['path'],sha256=item['sha256'],counts=result);save(out/'receipt.json',state)
        print(json.dumps(dict(profile=profile,actual_assets=10,external_passed=sum(v['counts']['passed'] for v in state['profiles'][profile]['integration_targets'].values())),ensure_ascii=False),flush=True)
    state['status']='hashing_current_archives_after_revalidation';save(out/'receipt.json',state)
    for n,v in asset_pins.items():assert sha(n)==v['sha256'] and metadata(Path(n))==initial_meta[n],n
    assert sound_metadata()==initial_sounds;state.update(status='executed_current_archive_native77_revalidation',active_profile=None,archive_inputs_unchanged=True,current_asset_report_sha256=sha(out/'current-asset-report.json'))
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],source_unchanged=True,new_sessions=0,new_casts=0),ensure_ascii=False),flush=True)
