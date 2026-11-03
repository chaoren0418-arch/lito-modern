"""Actual current75 memory sessions against the qualified normal Cargo test image, without source overlays."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time
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
ap=argparse.ArgumentParser();ap.add_argument('--cohort',type=Path,required=True);ap.add_argument('--profile',choices=['Debug','Release'],required=True);ap.add_argument('--qualification',type=Path,required=True);a=ap.parse_args()
build=a.cohort.resolve(strict=True);assert build.is_relative_to(V/'native-ability-finite-normal')
b=read(build/'cohort.json');assert b['status']=='executed_ability_finite_normals_related';info=b['profiles'][a.profile]
qpath=a.qualification.resolve(strict=True);assert qpath.parent.parent==build;q=read(qpath)
assert q['status']=='independently_qualified_ability_finite_normal_related_scope'and q['profile']==a.profile and q['source_manifest_sha256']==b['source_manifest_sha256']
assert q['passed']==31 and q['new_graphics_verified']==11 and not q['source_overlays_used']
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1048
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin';harness=P/'tests/oracle/NativePeerHarness.java';harness_sha=sha(harness)
assert sha(jar)==sha(formal)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
normal=b
def guard():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for r in normal['profiles'].values():assert sha(r['normal']['path'])==r['normal']['sha256']
    for r in info['targets'].values():assert sha(r['path'])==r['sha256']
    assert sha(jar)==sha(formal)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
    assert sha(harness)==harness_sha
guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()
out=V/'native-ability-normal-live'/('live-'+stamp);out.mkdir(parents=True)
work=P/'target/current/native44FE-extra-finite-effects'/('live-'+stamp);work.mkdir(parents=True);classes=work/'classes';classes.mkdir();server_root=work/'server';server_root.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'NativePeerHarness.java').write_bytes(harness.read_bytes());(out/'live_probe.rs').write_bytes((P/'tests/native_ability_live.rs').read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(h):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
assets=Path(r'D:\lineage開發\0-客戶端')
def asset_metadata():
    paths=[p for p in assets.iterdir()if p.is_file()and p.suffix.upper()in {'.PAK','.IDX'}]+[assets/'launcher/TW13081901.pak']+list((assets/'Sound').glob('*.wav'))
    return {str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns)for p in paths}
before=asset_metadata();save(out/'asset-metadata-before.json',before)
assert not any(os.environ.get(n)for n in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'])
env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),LITO_ABILITY_BUFF_LIVE_ROOT=str(work))
state=dict(status='preparing_current75_ability_live',profile=a.profile,compile=str(build),cohort_sha256=sha(build/'cohort.json'),resource_qualification=str(qpath),resource_qualification_sha256=sha(qpath),
    source_manifest_sha256=b['source_manifest_sha256'],normal_source_tree_used=True,runtime=str(work),runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),
    reference_jar=str(jar),reference_jar_sha256=sha(jar),catalog=str(catalog),catalog_sha256=sha(catalog),harness_sha256=harness_sha,process_history=[],owned_processes=[],
    source_overlays_used=False,normal_product_modified=False,repository_fixture='Unmodified ordinary memory characters/books/equipment, profile melee; no SQL or runtime authority mutation.',actual_sessions=0,actual_cast_requests=0,successful_casts=0,normal_GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
children=[]
def refresh():
    for child,job,stdout,stderr in children:
        if child.poll()is not None:job['exit_code']=child.returncode;stdout.close();stderr.close()
    state['owned_processes']=[job for child,job,_,_ in children if child.poll()is None];save(out/'receipt.json',state)
def launch(command,label,cwd):
    stdout=(out/(label+'.stdout')).open('wb');stderr=(out/(label+'.stderr')).open('wb')
    child=subprocess.Popen(command,cwd=cwd,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,command=command);state['process_history'].append(job);children.append((child,job,stdout,stderr));refresh();return child
print('ABILITY_BUFF_LIVE='+str(out),flush=True)
try:
    jdk=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin')
    compiler=launch([str(jdk/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/'NativePeerHarness.java')],'javac',work)
    assert compiler.wait(timeout=60)==0,'canonical isolated harness compilation failed';refresh()
    state['harness_class_pins']={str(p):sha(p)for p in classes.rglob('*.class')};assert state['harness_class_pins']
    server=launch([str(jdk/'java.exe'),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+env['TMP'],'-Duser.dir='+str(server_root),'-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',str(classes)+os.pathsep+str(jar),'local.lito381.server.NativePeerHarness',str(server_root),'melee'],'server',server_root)
    end=time.monotonic()+8
    while not(server_root/'ready.json').exists():refresh();assert server.poll()is None;assert time.monotonic()<end,'8s server ready bound';time.sleep(.02)
    ready=read(server_root/'ready.json');assert ready['jar_sha256'].upper()==sha(jar)
    (work/'endpoint.txt').write_text('127.0.0.1:'+str(ready['port']),encoding='ascii')
    client=launch([info['targets']['native_ability_live']['path'],'--ignored','--exact','actual_two_ability_buffs_self_and_nonclan_target','--nocapture'],'client',work)
    state['status']='executing_current75_ability_live';refresh();end=time.monotonic()+225
    while client.poll()is None:refresh();assert server.poll()in [None,0];assert time.monotonic()<end,'225s existing peer wrapper bound';time.sleep(.05)
    refresh();assert client.returncode==0,'actual ability probe failed';assert server.wait(timeout=15)==0;refresh()
    server_result=read(server_root/'result.json');assert server_result['actual_client_sessions']==4 and server_result['memberships_empty']and server_result['authority_mutations']==0 and not server_result['sql_access']
    for role in ['a','b']:
        result=read(work/(role+'-result.json'));assert result['error']is None and len(result['pictures'])==2
        shutdown=read(work/(role+'-shutdown.json'));assert shutdown['controller_quiescent']and shutdown['renderer_quiescent']
    assert len(read(work/'b-result.json')['actions'])==4
    state.update(status='executed_current75_ability_live_pending_independent_qualification',actual_sessions=4,actual_cast_requests=4,successful_casts=2,MP_rejections=1)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    for child,job,_,_ in reversed(children):
        if child.poll()is None:
            assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];job['cleanup_stopped_original_owned_handle']=True;child.terminate();child.wait(timeout=3)
    refresh();guard();after=asset_metadata();save(out/'asset-metadata-after.json',after);state['asset_metadata_unchanged']=before==after;save(out/'receipt.json',state);assert before==after
print(json.dumps(dict(run=str(out),status=state['status'],actual_sessions=state['actual_sessions']),ensure_ascii=False))
