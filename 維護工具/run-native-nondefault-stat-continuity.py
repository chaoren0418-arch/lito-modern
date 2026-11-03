"""Actual ordinary75 same-endpoint sessions following the advertised rental service."""
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
ap=argparse.ArgumentParser();ap.add_argument('--compile',type=Path,required=True);ap.add_argument('--compile-session',type=int);ap.add_argument('--compile-terminal-chunk',required=True);ap.add_argument('--compile-exit-code',type=int,required=True);a=ap.parse_args()
assert (a.compile_session is None or a.compile_session>0)and a.compile_exit_code==0 and a.compile_terminal_chunk not in ['','pending','unknown']
build=a.compile.resolve(strict=True);assert build.is_relative_to(V/'native-nondefault-stat-continuity')
b=read(build/'receipt.json');assert b['status']=='compiled_normal_public_API_nondefault_stat_probe'and b['owned_process']is None and not b['runtime_overrides']and not b['source_overlays_used']and b['normal_source_bodies_unchanged']
assert all(row['exit_code']==0 and not row.get('cleanup_stopped_original_owned_handle')for row in b['process_history'])and len(b['process_history'])==1
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
assert b['source_manifest_sha256']==queue['current_product_identity']['source_manifest_sha256']
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1049
normal=read(Path(b['normal_cohort'])/'cohort.json');library=Path(b['library_cohort']);assert sha(library/'completion-observation.json')==b['library_completion_sha256']
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(jar)==sha(formal)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'and sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
harness=build/'NondefaultTeleportHarness.java';peer=build/'NativePeerHarness.java';fixture=build/'NondefaultFixture.java'
assert b['main_class']=='local.lito381.server.NondefaultTeleportHarness' and set(b['fixture_source_pins'])=={harness.name,peer.name,fixture.name}
for path in [harness,peer,fixture]:assert sha(path)==b['fixture_source_pins'][path.name]
pins={str(path):sha(path)for path in [jar,formal,catalog,harness,peer,fixture,Path(b['product']),build/'probe.rs',build/'receipt.json']}
for proof in b['resource_qualifications'].values():assert sha(proof['qualification'])==proof['qualification_sha256']

def guard():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in pins.items():assert sha(n)==h,n
    for item in b['libraries'].values():assert sha(item['path'])==item['sha256']
    for item in normal['profiles'].values():assert sha(item['normal']['path'])==item['normal']['sha256']
guard()
ids=','.join(str(row['pid'])for row in [b['runner']]+b['process_history'])
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
for actual in live:
    for row in [b['runner']]+b['process_history']:
        if actual['ProcessId']==row['pid']:assert abs((datetime.fromisoformat(actual['created'])-datetime.fromisoformat(row['creation_utc'])).total_seconds())>.001,'original compile must be terminal'
assert not any(os.environ.get(n)for n in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'])
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+b['profile'].lower();out=V/'native-nondefault-stat-continuity'/('run-'+stamp);out.mkdir(parents=True)
work=P/'target/current/native-nondefault-stat-continuity'/('run-'+stamp);work.mkdir(parents=True);classes=work/'classes';classes.mkdir()
for path in [harness,peer,fixture]:(out/path.name).write_bytes(path.read_bytes())
(out/'probe.rs').write_bytes((build/'probe.rs').read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'compile-terminal-CIM.json').write_bytes(raw)
save(out/'compile-terminal-observation.json',dict(original_session=a.compile_session,returned_without_persistent_session=a.compile_session is None,terminal_tool_chunk=a.compile_terminal_chunk,observed_terminal_exit_code=0,compile_receipt_sha256=sha(build/'receipt.json'),original_recorded_compile_processes_terminal=True,identities=[b['runner']]+b['process_history']))
product=work/'immutable-same-endpoint-rental.exe';product.write_bytes(Path(b['product']).read_bytes());assert sha(product)==b['product_sha256']
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
assets=Path(r'D:\lineage開發\0-客戶端')
def asset_metadata():
    paths=[p for p in assets.iterdir()if p.is_file()and p.suffix.upper()in {'.PAK','.IDX'}]+[assets/'launcher/TW13081901.pak']+list((assets/'Sound').glob('*.wav'))
    return{str(p):dict(bytes=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns)for p in paths}
before=asset_metadata();save(out/'asset-metadata-before.json',before)
state=dict(status='preparing_same_endpoint_advertised_rental_probe',profile=b['profile'],compile=str(build),compile_receipt_sha256=sha(build/'receipt.json'),source_manifest_sha256=b['source_manifest_sha256'],probe_sha256=b['probe_sha256'],runtime=str(work),reference_jar=str(jar),reference_jar_sha256=sha(jar),catalog=str(catalog),catalog_sha256=sha(catalog),harness_sha256=sha(harness),peer_harness_sha256=sha(peer),fixture_sha256=sha(fixture),initial_server_fields=14,wire_projection_fields=9,original_five_wire_claimed=False,runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),pins=pins,product=str(product),product_sha256=sha(product),process_history=[],owned_processes=[],source_overlays_used=False,normal_source_modified=False,normal_source_bodies_unchanged=True,diagnostic_API_extension=False,actual_sessions=0,normal_channel_continuation_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
env={n:v for n,v in os.environ.items()if not n.startswith('LITO_')};env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),LITO_TELEPORT_ROOT=str(work),LITO_TELEPORT_SCENARIO='same-endpoint',LITO_TELEPORT_REFERENCE_SHA=sha(jar).lower())
children=[]
def refresh():
    for child,job,stdout,stderr in children:
        if child.poll()is not None:job['exit_code']=child.returncode;stdout.close();stderr.close()
    state['owned_processes']=[job for child,job,_,_ in children if child.poll()is None];save(out/'receipt.json',state)
def launch(command,label,cwd):
    stdout=(out/(label+'.stdout')).open('wb');stderr=(out/(label+'.stderr')).open('wb')
    child=subprocess.Popen(command,cwd=cwd,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,command=command);state['process_history'].append(job);children.append((child,job,stdout,stderr));refresh();return child
print('SAME_ENDPOINT_RENTAL_RUN='+str(out),flush=True);save(out/'receipt.json',state)
try:
    jdk=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin')
    compiler=launch([str(jdk/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/'NativePeerHarness.java'),str(out/'NondefaultTeleportHarness.java'),str(out/'NondefaultFixture.java')],'javac',work)
    assert compiler.wait(timeout=60)==0,'preserve canonical fixture compiler failure';refresh()
    state['harness_class_pins']={str(p):sha(p)for p in classes.rglob('*.class')};assert state['harness_class_pins']
    server=launch([str(jdk/'java.exe'),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Xverify:all','-Djava.io.tmpdir='+env['TMP'],'-Duser.dir='+str(work),'-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',str(classes)+os.pathsep+str(jar),'local.lito381.server.NondefaultTeleportHarness',str(work),'same-endpoint'],'server',work)
    deadline=time.monotonic()+8
    while not(work/'ready.json').exists():refresh();assert server.poll()is None;assert time.monotonic()<deadline,'8s server ready bound';time.sleep(.02)
    ready=read(work/'ready.json');assert ready['same_endpoint']and ready['jar_sha256'].upper()==sha(jar)
    client=launch([str(product),'actual_two_players_nondefault_stats_same_endpoint_resume_advertised_rental_service','--exact','--ignored','--nocapture'],'client',P)
    state['status']='executing_same_endpoint_advertised_rental_probe';refresh();deadline=time.monotonic()+220
    while client.poll()is None:refresh();assert server.poll()in [None,0];assert time.monotonic()<deadline,'220s original handoff wrapper bound';time.sleep(.1)
    refresh();assert client.returncode==0,'preserve actual probe failure';assert server.wait(timeout=8)==0;refresh()
    result=read(work/'result.json');assert result['actual_client_sessions']==6 and result['authority_mutations']==0 and not result['sql_access']and not result['state_persistent']and result['memberships_empty']and result['staged_loadouts_empty']and result['authenticated_accounts_empty']
    assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',(out/'client.stdout').read_text('utf-8-sig'))==[('1','0','0','0','0')]
    for role in ['a','b']:assert read(work/(role+'-handoff.json'))['claimed_scope']=='nondefault_initial_directory_and_effective_stat_continuity_same_endpoint'
    state.update(status='executed_same_endpoint_advertised_rental_pending_independent_readback',actual_sessions=6)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    for child,job,_,_ in reversed(children):
        if child.poll()is None:
            assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];job['cleanup_stopped_original_owned_handle']=True;child.terminate();child.wait(timeout=3)
    refresh()
    if(work/'result.json').exists():
        observed=read(work/'result.json');state['actual_sessions']=observed['actual_client_sessions'];state['server_result_sha256']=sha(work/'result.json')
    guard();after=asset_metadata();save(out/'asset-metadata-after.json',after);state['asset_metadata_unchanged']=before==after;save(out/'receipt.json',state);assert before==after
print(json.dumps(dict(run=str(out),status=state['status'],actual_sessions=state['actual_sessions']),ensure_ascii=False))
