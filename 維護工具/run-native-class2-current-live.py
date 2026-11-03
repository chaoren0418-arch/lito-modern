"""Owned C02 two-peer controller/Presentation diagnostic; no SQL or runtime authority edits."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['class2-world-entry'],required=True);ap.add_argument('--maxcasts',type=int,default=2);ap.add_argument('--compile',type=Path,required=True);a=ap.parse_args();assert 2<=a.maxcasts<=20
normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950';c=read(normal/'cohort.json')
assert c['status']=='executed_class2_normals_related' and c['owned_process'] is None
for p in ['Debug','Release']:
    q=read(next(normal.glob(p+'-qualified-*/qualified.json')));assert q['passed']==57 and q['source_manifest_sha256']==c['source_manifest_sha256']
build=a.compile.resolve(strict=True);assert build.is_relative_to(ROOT/'驗證紀錄/current/native-class2-current');b=read(build/'receipt.json');assert b['status']=='compiled_current_class2_source_diagnostic' and b['source_manifest_sha256']==c['source_manifest_sha256']
assert sha(b['normal_product'])==b['normal_product_sha256']
exe=Path(b['product']);assert sha(exe)==b['product_sha256']
inputs=read(build/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
fixture=ROOT/'驗證紀錄/current/native-class2-world-fixture/20261018-004101-764869';f=read(fixture/'receipt.json');fq=next(fixture.glob('qualified-*/qualified.json'));assert read(fq)['status']=='independently_qualified_native77_initial_memory_fixtures_only'
for n,h in f['class_pins'].items():assert sha(n)==h
jar=Path(f['reference_jar']);catalog=Path(f['common_catalog']['path']);assert sha(jar)==f['reference_jar_sha256'] and sha(catalog)==f['common_catalog']['sha256']
entry=Path(b['production_inputs']['diagnostic_probe.rs']['path']);assert entry.name=='live_probe_same_map.rs' and sha(entry)==b['production_inputs']['diagnostic_probe.rs']['sha256']
for name,pin in b['production_inputs'].items():assert sha(pin['path'])==pin['sha256']
broad=ROOT/'驗證紀錄/current/default-native-regression/20261017-221209-558-Release/receipt.json';from native1030_broad_gate import require_idle_runtime;require_idle_runtime(ROOT)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-class2-current-live'/stamp;out.mkdir(parents=True);runtime=PROJECT/'target/current/native-class2-same-map'/stamp;runtime.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'probe.rs').write_bytes((build/'diagnostic_probe.rs').read_bytes());(out/'NativeSelectorHarness.java').write_bytes((fixture/'NativeSelectorHarness.java').read_bytes())
assets=Path(r'D:/lineage開發/0-客戶端')
def asset_metadata():return {str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in assets.iterdir() if p.is_file() and p.suffix.upper() in {'.PAK','.IDX'}}|{str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in [assets/'launcher/TW13081901.pak',assets/'Sound/34.wav']}
before=asset_metadata();save(out/'asset-metadata-before.json',before)
state=dict(status='prepared_owned_native77_live_diagnostic',output=str(out),runtime=str(runtime),profile=a.profile,maxcasts=a.maxcasts,source_manifest_sha256=c['source_manifest_sha256'],normal_cohort=str(normal),production_inputs=b['production_inputs'],diagnostic_candidate_only=False,normal_GUI_product_exercised=False,normal_product=b["normal_product"],normal_product_sha256=b["normal_product_sha256"],controlled_visual_night_fixture=entry.name=='live_probe_same_map.rs',compile=str(build),compile_receipt_sha256=sha(build/'receipt.json'),product=str(exe),product_sha256=sha(exe),fixture_qualification=str(fq),fixture_qualification_sha256=sha(fq),reference_jar=str(jar),reference_jar_sha256=sha(jar),common_catalog=str(catalog),common_catalog_sha256=sha(catalog),harness_class_pins=f['class_pins'],process_history=[],owned_processes=[],actual_sessions=0,actual_casts=0,real_audio_device_opened=False,GUI_verified=False,whole_goal_complete=False)
env={k:v for k,v in os.environ.items() if k not in {'JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'}};env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetCurrentProcess.restype=wintypes.HANDLE
def created(h):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32);return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'original-runner-process.json',dict(pid=os.getpid(),creation_utc=created(k.GetCurrentProcess())))
started=[]
def refresh():
    for p,j,stdout,stderr in started:
        code=p.poll()
        if code is not None:j['exit_code']=code;stdout.close();stderr.close()
    state['owned_processes']=[j for p,j,_,_ in started if p.poll() is None];save(out/'receipt.json',state)
def launch(args,role):
    d=out/role;d.mkdir();stdout=(d/'stdout.log').open('wb');stderr=(d/'stderr.log').open('wb');p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    j=dict(pid=p.pid,creation_utc=created(wintypes.HANDLE(int(p._handle))),command=args,role=role);state['process_history'].append(j);started.append((p,j,stdout,stderr));refresh();return p
print('CLASS2_LIVE='+str(out),flush=True);save(out/'receipt.json',state)
try:
    java=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/java.exe')
    server=launch([str(java),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+env['TMP'],'-Duser.dir='+str(runtime),'-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',f['classes']+os.pathsep+str(jar),'local.lito381.server.NativeSelectorHarness',str(runtime),a.profile],'server')
    until=time.monotonic()+15
    while not (runtime/'ready.json').exists():
        refresh();assert server.poll() is None,'server exited before ready';assert time.monotonic()<until,'existing 15s readiness bound';time.sleep(.02)
    ready=read(runtime/'ready.json');assert ready['jar_sha256'].upper()==sha(jar)
    peers=[launch([str(exe),'127.0.0.1:'+str(ready['port']),str(runtime),role,a.profile,str(a.maxcasts)],role) for role in ['a','b','c']]
    state['status']='executing_owned_native77_controller_presentation';refresh();until=time.monotonic()+225
    while any(p.poll() is None for p in peers):
        refresh();assert all(p.poll() in [None,0] for p in peers),'peer failed; preserve diagnosis';assert server.poll() in [None,0],'server failed';assert time.monotonic()<until,'existing 225s wrapper bound';time.sleep(.05)
    assert all(p.returncode==0 for p in peers);server.wait(timeout=15);refresh();assert server.returncode==0
    reports={role:read(runtime/(role+'-result.json')) for role in ['a','b']};server_report=read(runtime/'result.json')
    assert server_report['actual_client_sessions']==6 and server_report['memberships_empty'] and server_report['authority_mutations']==0 and not server_report['sql_access']
    for role,report in reports.items():
        assert report['after']['world']['map_id']==4 and report['old_epoch']==report['new_epoch']
        assert report['old_surface_rejected'] and report['old_cue_deliveries']==0
        assert report['source34_sink_calls']==0
        same=report['same_map'];assert same['before']['world']['map_id']==same['after']['world']['map_id']==4
        assert same['after']['world']['generation']==same['before']['world']['generation']+1
        assert same['held_cue_outcomes']==[dict(first='Stale',duplicate='AlreadyConsumed')]
        assert len(report['held_cue_outcomes'])==1 and report['held_cue_outcomes'][0]['first'] in ['Stale','AlreadyConsumed'] and report['held_cue_outcomes'][0]['duplicate']=='AlreadyConsumed'
        shutdown=read(runtime/(role+'-shutdown.json'));assert shutdown['error'] is None and shutdown['close_error'] is None and shutdown['client_quiescent'] and shutdown['view_quiescent']
    third=read(runtime/'c-result.json');assert third['natural_battle_result'] and third['frame_retired']
    third_close=read(runtime/'c-shutdown.json');assert third_close['error'] is None and third_close['close_error'] is None and third_close['client_quiescent'] and third_close['view_quiescent']
    assert reports['a']['target_motion']['verified'] if a.maxcasts==3 else reports['a']['source_active_effects']>0
    state.update(status='executed_class2_WORLD_retirement',actual_sessions=6,actual_requests=11,actual_casts=1,actual_world_tree_transitions=4,graphics_observed=[11736],canonical_graphics_observed=True,server_result=server_report,source_ambient_counter=reports['a']['source_ambient_counter'])
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    for p,j,stdout,stderr in reversed(started):
        if p.poll() is None:
            assert created(wintypes.HANDLE(int(p._handle)))==j['creation_utc'];j['cleanup_stopped_original_owned_handle']=True;p.terminate();p.wait(timeout=3)
    refresh();after=asset_metadata();save(out/'asset-metadata-after.json',after);state['asset_metadata_unchanged']=before==after
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    assert sha(exe)==state['product_sha256'] and sha(jar)==state['reference_jar_sha256'] and sha(catalog)==state['common_catalog_sha256'];assert before==after,'asset input changed; retain failure'
    state['broad_still_compiling_after_runtime']=read(broad)['status']=='compiling_default_targets'
    state['all_started_processes_terminal']=all(p.poll() is not None for p,_,_,_ in started);save(out/'receipt.json',state)
print(json.dumps({k:state[k] for k in ['output','status','actual_sessions','actual_casts','graphics_observed','canonical_graphics_observed']},ensure_ascii=False),flush=True)
