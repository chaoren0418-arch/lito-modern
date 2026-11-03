"""Prepare an initial-stat fixture; compile/describe only after current full library."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';P=R/'LiTo381-Modern';T=R/'維護工具'
JDK=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('mode',choices=['prepare','compile-describe']);ap.add_argument('input',type=Path);a=ap.parse_args()
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256']
assert source=='AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D'
if a.mode=='prepare':
    normal=a.input.resolve(strict=True);assert normal.is_relative_to(V/'native-keyboard-focus-normal')
    q=read(normal/'qualified/qualified.json');assert q['source_manifest_sha256']==source and q['recorded_owned_processes_all_terminal']
    inputs=read(normal/'compiled-inputs.json')
    for n,h in inputs.items():assert sha(R/n)==h,n
    origin=T/'candidates/native-teleport-fixture-arm/NativeTeleportHarness.java';peer=P/'tests/oracle/NativePeerHarness.java'
    fixture=T/'candidates/native-nondefault-character-fixture/NondefaultFixture.java'
    assert sha(origin)=='DCF1A5BC267C149908E61C6C144BFA0F169F7AC1D67F1D75E72BD6E6424AD869'
    out=V/'native-nondefault-character-fixture'/('prepared-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
    body=origin.read_text('utf-8-sig');replacements=[
        ('public final class NativeTeleportHarness {','public final class NondefaultTeleportHarness {'),
        ('var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試乙",memoryBacking);',
         'var initialSource=NondefaultFixture.accounts(memoryBacking);\n            NativePeerHarness.write(root.resolve("initial-source-characters.json"),"{\\\"PeerFixture\\\":"+NondefaultFixture.rowsJson(initialSource,"PeerFixture")+",\\\"PeerFixture2\\\":"+NondefaultFixture.rowsJson(initialSource,"PeerFixture2")+"}");\n            var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試乙",initialSource);')]
    for before,after in replacements:assert body.count(before)==1;body=body.replace(before,after)
    (out/'NondefaultTeleportHarness.java').write_text(body,encoding='utf-8',newline='\n')
    for f in [peer,fixture]:(out/f.name).write_bytes(f.read_bytes())
    (out/'preparer.py').write_bytes(Path(__file__).read_bytes())
    save(out/'prepared.json',dict(status='prepared_nondefault_initial_stat_sources_only',source_manifest_sha256=source,
        compiled_inputs=inputs,normal_qualification=str(normal/'qualified/qualified.json'),
        fixture_origins={str(f):sha(f)for f in [origin,peer,fixture]},fixture_sources={f.name:sha(f)for f in out.glob('*.java')},
        fixture_replacements=replacements,compiler_or_runtime_started=False,initial_stat_fields_per_character=14,
        handler_or_clock_or_authority_changes=False,stat_continuity_verified=False,GUI_verified=False,whole_goal_complete=False))
    print(json.dumps(dict(prepared=str(out),compiler_or_runtime_started=False),ensure_ascii=False));raise SystemExit(0)
out=a.input.resolve(strict=True);assert out.is_relative_to(V/'native-nondefault-character-fixture')
p=read(out/'prepared.json');assert p['source_manifest_sha256']==source
assert w['current_full_default_library_verified']and not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
for n,h in p['compiled_inputs'].items():assert sha(R/n)==h,n
for n,h in p['fixture_sources'].items():assert sha(out/n)==h,n
for n,h in p['fixture_origins'].items():assert sha(n)==h,n
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';assert sha(jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
stage=out/'compiled-description';stage.mkdir();classes=stage/'classes';classes.mkdir()
(stage/'preparer.py').write_bytes(Path(__file__).read_bytes())
env={n:v for n,v in os.environ.items()if not n.startswith('LITO_')};env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    times=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in times])
    ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='preparing_initial_memory_description_only',pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),
    prepared=str(out),source_manifest_sha256=source,processes=[],owned_process=None,actual_network_sessions=0,stat_continuity_verified=False)
save(stage/'cohort.json',state)
commands=[('compiler',[str(JDK/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(jar),'-d',str(classes)]+[str(out/n)for n in p['fixture_sources']]),
    ('description',[str(JDK/'java.exe'),'-Dfile.encoding=UTF-8','-Dlito.owned.root='+str(R),'-cp',str(classes)+os.pathsep+str(jar),'local.lito381.server.NondefaultFixture',str(stage/'description.json')])]
for label,command in commands:
    with(stage/(label+'.stdout.log')).open('wb')as stdout,(stage/(label+'.stderr.log')).open('wb')as stderr:
        child=subprocess.Popen(command,cwd=stage,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        handle=wintypes.HANDLE(int(child._handle));row=dict(role=label,pid=child.pid,creation_utc=birth(handle),command=command,exit_code=None)
        state['processes'].append(row);state.update(status='executing_'+label,owned_process=row);save(stage/'cohort.json',state)
        try:code=child.wait()
        except BaseException:
            if child.poll()is None:
                assert birth(handle)==row['creation_utc'];child.kill();child.wait();row['cleanup_stopped_original_owned_handle']=True
            state.update(status='failed_preserved');raise
        finally:
            row.update(exit_code=child.returncode,original_retained_handle_terminal=child.poll()is not None)
            state['owned_process']=None;save(stage/'cohort.json',state)
    save(stage/(label+'-exit.json'),row)
    if code!=0:state['status']='failed_preserved';save(stage/'cohort.json',state)
    assert code==0,'Retain candidate compiler/initial-description failure; no gameplay was started'
description=read(stage/'description.json');assert description['status']=='initial_memory_fixture_description_verified_only'
assert description['network_sessions']==0 and not description['sql_access']and not description['stat_continuity_verified']
for n,h in p['compiled_inputs'].items():assert sha(R/n)==h,n
state['status']='verified_initial_memory_description_only';save(stage/'cohort.json',state)
pins={str(f.relative_to(stage)):sha(f)for f in stage.rglob('*')if f.is_file()}
with zipfile.ZipFile(stage/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in pins:z.write(stage/n,n)
with zipfile.ZipFile(stage/'raw.zip')as z:assert set(z.namelist())==set(pins)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(stage/'readback.json',dict(status='candidate_initial_memory_description_only',prepared=str(out),prepared_sha256=sha(out/'prepared.json'),
    raw_archive_sha256=sha(stage/'raw.zip'),raw_files=pins,field_assertions=description['field_assertions'],
    actual_network_sessions=0,stat_continuity_verified=False,GUI_verified=False,whole_goal_complete=False))
print(json.dumps(dict(description=str(stage/'readback.json'),field_assertions=description['field_assertions'],stat_continuity_verified=False),ensure_ascii=False))
