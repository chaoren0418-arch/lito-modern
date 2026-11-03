"""Execute isolated resource integration; never claims server casts or normal GUI."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,time
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--compile',type=Path,required=True);a=ap.parse_args()
build=a.compile.resolve(strict=True);assert build.is_relative_to(V/'native44FE-extra-finite-effects')
b=read(build/'receipt.json');assert b['status']=='compiled_isolated_lazy_finite_effect_variant'
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
assert b['source_overlays_used']and not b['normal_product_modified']
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041
normal=read(Path(b['normal_qualification']).parent.parent/'cohort.json')
def guard():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for r in normal['profiles'].values():assert sha(r['normal']['path'])==r['normal']['sha256']
    for r in b['products'].values():assert sha(r['path'])==r['sha256']
    assert sha(b['diagnostic_library'])==b['diagnostic_library_sha256']
guard()
component=ROOT/'維護工具/candidates/native44FE-extra-skill-finite-effects';spec=read(component/'preparation.json')
definition=component/'native_additional_finite_effects.rs';assert spec['candidate_sha256']==sha(definition)
assert (build/'native_additional_finite_effects.rs').read_bytes().startswith(definition.read_bytes()),'original finite-definition guard must stay exact'
assert sha(spec['original_metadata'])==spec['original_metadata_sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+b['profile'].lower()
out=V/'native44FE-extra-finite-effects'/('run-'+stamp);out.mkdir()
work=P/'target/current/native44FE-extra-finite-effects'/('run-'+stamp);work.mkdir(parents=True)
(work/'original-effect-definitions.json').write_bytes(Path(spec['original_metadata']).read_bytes());save(work/'candidate-graphics.json',spec['graphics'])
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
(out/'original-effect-definitions.json').write_bytes(Path(spec['original_metadata']).read_bytes());save(out/'candidate-graphics.json',spec['graphics'])
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
assets=Path(r'D:\lineage開發\0-客戶端')
def asset_metadata():
    paths=[p for p in assets.iterdir()if p.is_file()and p.suffix.upper()in {'.PAK','.IDX'}]+[assets/'launcher/TW13081901.pak']+list((assets/'Sound').glob('*.wav'))
    return {str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns)for p in paths}
before=asset_metadata();save(out/'asset-metadata-before.json',before)
state=dict(status='executing_isolated_finite_resource_tests',profile=b['profile'],compile=str(build),compile_receipt_sha256=sha(build/'receipt.json'),runtime=str(work),
    runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,source_manifest_sha256=b['base_source_manifest_sha256'],candidate_manifest_sha256=b['candidate_manifest_sha256'],
    source_overlays_used=True,normal_product_modified=False,actual_sessions=0,actual_casts=0,synthetic_S37_inputs=True,normal_GUI_verified=False,audio_device_playback_verified=False,whole_goal_complete=False)
env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),LITO_EXTRA_FINITE_RENDER_OUT=str(work))
print('EXTRA_FINITE_RUN='+str(out),flush=True);save(out/'receipt.json',state)
try:
    counts={'scene23':23,'additional11':1}
    if b.get('ability_request_candidate_included'):counts.update(ability5=5,ability_world2=2)
    for label,count in counts.items():
        guard();command=[b['products'][label]['path'],'--test-threads=1','--nocapture']
        with(out/(label+'.stdout')).open('wb')as stdout,(out/(label+'.stderr')).open('wb')as stderr:
            child=subprocess.Popen(command,cwd=work,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=label);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=225)
            except subprocess.TimeoutExpired:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];job['cleanup_stopped_original_owned_handle']=True;child.terminate();job['exit_code']=child.wait(timeout=3);raise
            finally:
                state['owned_process']=None;save(out/'receipt.json',state)
        assert job['exit_code']==0,label
        text=(out/(label+'.stdout')).read_text('utf-8-sig');rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
        assert rows==[(str(count),'0','0','0','0')],(label,rows)
    result=read(work/'resource-result.json');assert result['status']=='candidate_resource_render_and_source_event_progression_passed'
    assert sorted(r['graphics']for r in result['cases'])==spec['graphics']and len(result['cases'])==11
    state['status']='executed_candidate_resources_pending_independent_qualification'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    guard();after=asset_metadata();save(out/'asset-metadata-after.json',after);state['asset_metadata_unchanged']=before==after;save(out/'receipt.json',state);assert before==after
print(json.dumps(dict(run=str(out),status=state['status'],normal_product_modified=False),ensure_ascii=False))
