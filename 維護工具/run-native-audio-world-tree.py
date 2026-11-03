"""Two isolated ordinary C02 servers; explicit user reconnect audio diagnostic."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(p.read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--compile',type=Path,required=True);a=ap.parse_args()
build=a.compile.resolve(strict=True);assert build.is_relative_to(ROOT/'驗證紀錄/current/native-audio-world-tree')
b=read(build/'receipt.json');assert b['status']=='compiled_current_production_Presentation_test'
inputs=read(build/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
exe=Path(b['product']);assert sha(exe)==b['product_sha256']
fixture=ROOT/'驗證紀錄/current/native77-live-fixture/20261017-190314-200399';f=read(fixture/'receipt.json')
for n,h in f['class_pins'].items():assert sha(n)==h,n
jar=Path(f['reference_jar']);catalog=Path(f['common_catalog']['path'])
assert sha(jar)==f['reference_jar_sha256'] and sha(catalog)==f['common_catalog']['sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-audio-world-tree'/('run-'+stamp);out.mkdir()
runtime=PROJECT/'target/current/native-audio-world-tree'/stamp;runtime.mkdir(parents=True)
for name in ['source']:(runtime/name).mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'probe.rs').write_bytes((build/'probe.rs').read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetCurrentProcess.restype=wintypes.HANDLE
def created(h):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'runner-identity.json',dict(pid=os.getpid(),creation_utc=created(k.GetCurrentProcess())))
assets=Path(r'D:/lineage開發/0-客戶端')
def asset_metadata():
    paths=[p for p in assets.iterdir() if p.is_file() and p.suffix.upper() in {'.PAK','.IDX'}]+[assets/'launcher/TW13081901.pak']+list((assets/'Sound').glob('*.wav'))
    return {str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns) for p in paths}
before=asset_metadata();save(out/'asset-metadata-before.json',before)
state=dict(status='prepared',output=str(out),runtime=str(runtime),profile=b['profile'],compile=str(build),compile_receipt_sha256=sha(build/'receipt.json'),product=str(exe),product_sha256=sha(exe),source_manifest_sha256=b['source_manifest_sha256'],reference_jar=str(jar),reference_jar_sha256=sha(jar),catalog=str(catalog),catalog_sha256=sha(catalog),harness_class_pins=f['class_pins'],process_history=[],owned_processes=[],actual_sessions=0,scope='same GAME connection arena→WORLD skill131 with real deferred heal cue, character/items and movement; not C72 endpoint handoff',GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
env={n:v for n,v in os.environ.items() if n not in {'JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'}}
env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),LITO_AUDIO_WORLD_TREE_ROOT=str(runtime))
started=[]
def refresh():
    for p,j,stdout,stderr in started:
        if p.poll() is not None:j['exit_code']=p.returncode;stdout.close();stderr.close()
    state['owned_processes']=[j for p,j,_,_ in started if p.poll() is None];save(out/'receipt.json',state)
def launch(args,role,cwd):
    d=out/role;d.mkdir();stdout=(d/'stdout.log').open('wb');stderr=(d/'stderr.log').open('wb')
    p=subprocess.Popen(args,cwd=cwd,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    j=dict(pid=p.pid,creation_utc=created(wintypes.HANDLE(int(p._handle))),command=args,role=role)
    started.append((p,j,stdout,stderr));state['process_history'].append(j);refresh();return p
print('AUDIO_RECONNECT_RUN='+str(out),flush=True)
try:
    servers=[];ready={};java=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/java.exe')
    for name in ['source']:
        work=runtime/name
        server=launch([str(java),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+env['TMP'],'-Duser.dir='+str(work),'-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',f['classes']+os.pathsep+str(jar),'local.lito381.server.NativeSelectorHarness',str(work),'world-entry'],name,work);servers.append(server)
        until=time.monotonic()+15
        while not (work/'ready.json').exists():
            refresh();assert server.poll() is None,'server exited';assert time.monotonic()<until,'15s ready bound';time.sleep(.02)
        r=read(work/'ready.json');assert r['jar_sha256'].upper()==sha(jar);ready[name]='127.0.0.1:'+str(r['port'])
    save(runtime/'endpoints.json',ready)
    client=launch([str(exe),'--ignored','--exact','actual_battle_audio_world_tree_same_connection','--nocapture'],'client',runtime)
    state['status']='executing';refresh();until=time.monotonic()+225
    while client.poll() is None:
        refresh();assert all(p.poll() in [None,0] for p in servers),'server failed';assert time.monotonic()<until,'225s wrapper bound';time.sleep(.05)
    refresh();assert client.returncode==0,'client diagnostic failed'
    for server in servers:server.wait(timeout=15)
    refresh();assert all(p.returncode==0 for p in servers)
    reports={n:read(runtime/n/'result.json') for n in ['source']}
    assert all(j['actual_client_sessions']==4 and j['memberships_empty'] and j['authority_mutations']==0 and not j['sql_access'] for j in reports.values())
    for role in ['a','b']:
        j=read(runtime/(role+'-result.json'));assert j['old_cue_deliveries']==0 and j['old_surface_rejected']
        assert j['held_cue_outcomes']==[dict(first='Stale',duplicate='AlreadyConsumed')]
        s=read(runtime/(role+'-shutdown.json'));assert s['client_quiescent'] and s['view_quiescent'] and s['error'] is None and s['close_error'] is None
    state.update(status='executed_diagnostic_pending_independent_qualification',actual_sessions=4,server_reports=reports)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    for p,j,stdout,stderr in reversed(started):
        if p.poll() is None:
            assert created(wintypes.HANDLE(int(p._handle)))==j['creation_utc'];j['cleanup_stopped_original_owned_handle']=True;p.terminate();p.wait(timeout=3)
    refresh();after=asset_metadata();save(out/'asset-metadata-after.json',after);state['asset_metadata_unchanged']=before==after
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    assert sha(exe)==state['product_sha256'] and sha(jar)==state['reference_jar_sha256'] and sha(catalog)==state['catalog_sha256']
    assert before==after,'asset metadata changed'
    state['all_started_processes_terminal']=all(p.poll() is not None for p,_,_,_ in started);save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],actual_sessions=state['actual_sessions']),ensure_ascii=False),flush=True)
