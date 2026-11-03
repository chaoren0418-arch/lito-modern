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
build=a.compile.resolve(strict=True);assert build.is_relative_to(ROOT/'驗證紀錄/current/native44FE-world-tree')
b=read(build/'receipt.json');assert b['status']=='compiled_current_production_Presentation_test'
inputs=read(build/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
exe=Path(b['product']);assert sha(exe)==b['product_sha256']
jar=PROJECT/'target/current/server-reference/formal-server-75DA69B0.jar';catalog=PROJECT/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(jar)==sha(formal)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert b['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0' and len(inputs)==1041
assert not b['source_overlays_used'] and not b['diagnostic_API_extension']
harness=PROJECT/'tests/oracle/NativePeerHarness.java';harness_sha=sha(harness)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native44FE-world-tree'/('run-'+stamp);out.mkdir()
runtime=PROJECT/'target/current/native44FE-world-tree'/stamp;runtime.mkdir(parents=True)
for name in ['source']:(runtime/name).mkdir()
classes=runtime/'classes';classes.mkdir()
(out/'NativePeerHarness.java').write_bytes(harness.read_bytes())
assert not any(os.environ.get(n)for n in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'])
jdk=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin')
with (out/'javac.stdout.log').open('wb')as stdout,(out/'javac.stderr.log').open('wb')as stderr:
    compiled=subprocess.run([str(jdk/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/'NativePeerHarness.java')],cwd=runtime,env=dict(os.environ,TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch')),stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
assert compiled.returncode==0,'owned canonical harness compile failed'
f=dict(classes=str(classes),class_pins={str(p):sha(p)for p in classes.rglob('*.class')})
assert f['class_pins'] and sha(harness)==harness_sha

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
state=dict(status='prepared',output=str(out),runtime=str(runtime),profile=b['profile'],compile=str(build),compile_receipt_sha256=sha(build/'receipt.json'),product=str(exe),product_sha256=sha(exe),source_manifest_sha256=b['source_manifest_sha256'],reference_jar=str(jar),reference_jar_sha256=sha(jar),catalog=str(catalog),catalog_sha256=sha(catalog),harness_class_pins=f['class_pins'],process_history=[],owned_processes=[],actual_sessions=0,scope='same GAME connection arena→WORLD skill131 with real deferred heal cue, character/items and movement; not C72 endpoint handoff',GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,normal_source_bodies_unchanged=True,source_overlays_used=False,diagnostic_API_extension=False,initial_repository_fixture='Initial skill131 added to both isolated memory character books; original class, stats, graphics, HP/MP retained; no runtime authority injection',canonical_harness_sha256=harness_sha)
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
        server=launch([str(java),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+env['TMP'],'-Duser.dir='+str(work),'-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',f['classes']+os.pathsep+str(jar),'local.lito381.server.NativePeerHarness',str(work),'world-entry'],name,work);servers.append(server)
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
    assert sha(formal)==sha(jar) and sha(harness)==harness_sha
    assert sha(exe)==state['product_sha256'] and sha(jar)==state['reference_jar_sha256'] and sha(catalog)==state['catalog_sha256']
    assert before==after,'asset metadata changed'
    state['all_started_processes_terminal']=all(p.poll() is not None for p,_,_,_ in started);save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],actual_sessions=state['actual_sessions']),ensure_ascii=False),flush=True)
