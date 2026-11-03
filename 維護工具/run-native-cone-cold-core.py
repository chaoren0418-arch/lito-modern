"""Execute frozen core scenarios only after the original full library is terminal."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--debug-build',type=Path,required=True);ap.add_argument('--release-build',type=Path,required=True);ap.add_argument('--library-run',type=Path,required=True);a=ap.parse_args()
lib=a.library_run.resolve(strict=True);assert lib.is_relative_to(ROOT/'驗證紀錄/current/native1034-library-default')
c=read(lib/'cohort.json');terminal=read(lib/'completion-observation.json')
assert c['status']=='executed_both_current_full_library_default_selections' and c['owned_process'] is None
assert terminal['observed_terminal_exit_code']==0 and terminal['recorded_original_owned_processes_all_terminal'] and terminal['cohort_sha256']==sha(lib/'cohort.json')
for profile in ['Debug','Release']:
    q=read(lib/profile/'full-default-qualified/qualified.json');assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['failed']==q['Rust_exit_code']==0
builds={'Debug':a.debug_build.resolve(strict=True),'Release':a.release_build.resolve(strict=True)};compiled={};source_pins=None
for profile,build in builds.items():
    assert build.is_relative_to(ROOT/'驗證紀錄/current/native-cone-cold-core');b=read(build/'receipt.json')
    assert b['status']=='compiled_isolated_cold_core_diagnostic' and b['compiler']['exit_code']==0 and b['profile']==profile and not b['source_overrides_used']
    assert b['source_manifest_sha256']==c['source_manifest_sha256'] and sha(b['product'])==b['product_sha256']
    pe=Path(b['product']).read_bytes();offset=int.from_bytes(pe[60:64],'little');assert pe[:2]==b'MZ' and pe[offset:offset+4]==b'PE\0\0' and int.from_bytes(pe[offset+4:offset+6],'little')==0x8664
    pins=read(build/'compiled-inputs.json');assert len(pins)==1035 and all(sha(ROOT/n)==h for n,h in pins.items())
    for v in b['candidate_inputs'].values():assert sha(v['path'])==v['sha256']
    same={n:v['sha256']for n,v in b['candidate_inputs'].items()};assert source_pins is None or source_pins==same;source_pins=same
    if profile=='Release':
        for flag in ['opt-level=z','lto=thin','codegen-units=16','strip=symbols']:assert flag in b['compiler']['command']
    compiled[profile]=b
out=ROOT/'驗證紀錄/current/native-cone-cold-core-runtime'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(handle)),*[ctypes.byref(t)for t in ts]);v=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=v//10)).isoformat()
state=dict(status='prepared_frozen_cold_core_scenarios',source_manifest_sha256=c['source_manifest_sha256'],current_library_completion=str(lib/'completion-observation.json'),current_library_completion_sha256=sha(lib/'completion-observation.json'),builds={p:dict(path=str(builds[p]),receipt_sha256=sha(builds[p]/'receipt.json'),product_sha256=b['product_sha256'])for p,b in compiled.items()},candidate_inputs=source_pins,original_runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,profiles={},synthetic_local_projectile_cues=True,normal_product_registered=False,new_casts=0,new_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'receipt.json',state);print('COLD_CORE_RUNTIME='+str(out),flush=True)
env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
expected=['entry_immediate_dispatch','captured_flight_arrival_current_anchor','frozen_impact_late_from_now','terminal_source0_full_dwell','old_owner_launch_and_unplayed_audio_rejected','reused_target_rejects_impact','same_map_new_generation_retires_flight']
try:
    for profile,b in compiled.items():
        d=out/profile;d.mkdir();report=d/'scenarios.json';command=[b['product'],r'D:\lineage開發\0-客戶端',str(report)]
        with (d/'stdout.log').open('wb')as stdout,(d/'stderr.log').open('wb')as stderr:
            child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);job=dict(pid=child.pid,creation_utc=birth(child._handle),command=command);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=120)
            except subprocess.TimeoutExpired:
                assert birth(child._handle)==job['creation_utc'];child.terminate();job['exit_code']=child.wait(timeout=3);job['timed_out']=True;state['owned_process']=None;raise
        state['owned_process']=None;save(out/'receipt.json',state);assert job['exit_code']==0,'preserve the first actual scenario failure'
        s=read(report);assert s['status']=='executed_isolated_cold_core_scenarios_only' and [v['case']for v in s['cases']]==expected
        assert s['synthetic_local_projectile_cues'] and s['actual_C02_casts']==s['new_sessions']==0 and not s['rendering_executed']
        state['profiles'][profile]=dict(cases=7,report=str(report),report_sha256=sha(report),actual_exit_code=0);save(out/'receipt.json',state)
    assert read(out/'Debug/scenarios.json')==read(out/'Release/scenarios.json'),'codegen variants produced different core outcomes'
    state['status']='executed_both_profiles_frozen_cold_core_scenarios_only'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    for n,h in pins.items():assert sha(ROOT/n)==h
    for b in compiled.values():
        assert sha(b['product'])==b['product_sha256']
        for v in b['candidate_inputs'].values():assert sha(v['path'])==v['sha256']
    save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),cases_each=7,actual_C02_casts=0,normal_product_registered=False),ensure_ascii=False),flush=True)
