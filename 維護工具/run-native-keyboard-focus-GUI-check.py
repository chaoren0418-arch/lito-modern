"""Prepare/run only a bounded normal-window focus regression, never full GUI acceptance."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,shutil,subprocess,time
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';P=R/'LiTo381-Modern';JDK=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,d):Path(p).write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|ts[0].dwHighDateTime<<32
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
ap=argparse.ArgumentParser();ap.add_argument('mode',choices=['prepare','run']);ap.add_argument('input',type=Path);ap.add_argument('--profile',choices=['Debug','Release']);a=ap.parse_args()
queue=read(R/'工作佇列.json');assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
env=os.environ.copy();env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
if a.mode=='prepare':
    normal=a.input.resolve(strict=True);assert normal.is_relative_to(V/'native-keyboard-focus-normal')
    qp=normal/'qualified/qualified.json';q=read(qp);assert q['status']=='independently_qualified_keyboard_focus_normal_related_both_profiles'
    assert q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_all_terminal']and q['passed_per_profile']==16
    assert q['source_manifest_sha256']==queue['current_product_identity']['source_manifest_sha256']
    inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1048
    for n,h in inputs.items():assert sha(R/n)==h,n
    out=V/'native-keyboard-focus-sky-gui'/('prepared-focus-only-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
    jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
    assert sha(jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
    assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
    origin=P/'tests/oracle/NativePeerHarness.java';body=origin.read_text('utf-8-sig')
    replacements=[('String profile=args.length>1?args[1]:"melee";','boolean focusSingle=args.length>1&&args[1].equals("gui-focus-single");\n        String profile=focusSingle?"rental-entry":args.length>1?args[1]:"melee";'),
        ('int peerCount=profile.equals("siege-native")?30:', 'int peerCount=focusSingle?1:profile.equals("siege-native")?30:'),
        ('login.setSoTimeout(45000);game.setSoTimeout(45000);','login.setSoTimeout(focusSingle?180000:45000);game.setSoTimeout(focusSingle?180000:45000);')]
    for before,after in replacements:assert body.count(before)==1;body=body.replace(before,after)
    source=out/'NativePeerHarness.java';source.write_text(body,encoding='utf-8',newline='\n');classes=out/'classes';classes.mkdir()
    with(out/'javac.stdout.log').open('wb')as stdout,(out/'javac.stderr.log').open('wb')as stderr:
        compiler=subprocess.Popen([str(JDK/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(source)],cwd=out,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        compiler_identity=dict(pid=compiler.pid,creation_utc=birth(wintypes.HANDLE(int(compiler._handle))));code=compiler.wait()
    assert code==0,'Retain fixture compiler failure'
    products={}
    for profile in ['Debug','Release']:
        product=q['profiles'][profile]['normal'];target=out/profile/'LiToModern-FocusCheck.exe';target.parent.mkdir()
        assert sha(product['path'])==product['sha256'];shutil.copyfile(product['path'],target);assert sha(target)==product['sha256']
        products[profile]={'a':dict(path=str(target),original_path=product['path'],sha256=product['sha256'])}
    save(out/'prepared.json',dict(scope='normal_focus_regression_only',normal_qualification=str(qp),normal_qualification_sha256=sha(qp),
        source_manifest_sha256=q['source_manifest_sha256'],compiled_inputs=inputs,jar=str(jar),jar_sha256=sha(jar),catalog=str(catalog),catalog_sha256=sha(catalog),
        classes=str(classes),class_files={str(p.relative_to(classes)):sha(p)for p in classes.rglob('*.class')},fixture_sources={str(source):sha(source)},
        fixture_origin=str(origin),fixture_origin_sha256=sha(origin),fixture_replacements=replacements,products=products,compiler=compiler_identity,compiler_exit_code=0,
        authority_mutations=0,normal_GUI_qualified=False,full_library_scope_qualified=False,whole_goal_complete=False))
    print(json.dumps(dict(prepared=str(out),scope='normal_focus_regression_only'),ensure_ascii=False));raise SystemExit(0)
prepared=a.input.resolve(strict=True);p=read(prepared/'prepared.json');assert p['scope']=='normal_focus_regression_only'and a.profile
assert sha(p['normal_qualification'])==p['normal_qualification_sha256']and read(p['normal_qualification'])['source_manifest_sha256']==queue['current_product_identity']['source_manifest_sha256']
for n,h in p['compiled_inputs'].items():assert sha(R/n)==h,n
for n,h in p['class_files'].items():assert sha(Path(p['classes'])/n)==h,n
assert sha(p['jar'])==p['jar_sha256']and sha(p['catalog'])==p['catalog_sha256']
out=V/'native-keyboard-focus-sky-gui'/('case-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-focus-only');out.mkdir()
state=dict(status='preparing_owned_server',prepared=str(prepared),prepared_sha256=sha(prepared/'prepared.json'),source_manifest_sha256=p['source_manifest_sha256'],
    profile=a.profile,scenario='focus-regression-only',automation_scope='accessibility-keyboard-focus-regression',pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),processes=[],
    GUI_verified=False,whole_goal_complete=False,full_library_qualification_inferred=False)
persist=lambda:save(out/'cohort.json',state);children=[]
def launch(command,role):
    stdout=(out/(role+'.stdout.log')).open('wb');stderr=(out/(role+'.stderr.log')).open('wb')
    child=subprocess.Popen(command,cwd=out,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    row=dict(role=role,pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,exit_code=None);state['processes'].append(row);children.append((child,row,stdout,stderr));persist();return child
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    server=launch([str(JDK/'java.exe'),'-Dfile.encoding=UTF-8','-Xverify:all','-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true',
        '-Dlito.commonCatalog='+p['catalog'],'-cp',p['classes']+';'+p['jar'],'local.lito381.server.NativePeerHarness',str(out),'gui-focus-single'],'server')
    deadline=time.monotonic()+8
    while not(out/'ready.json').exists():assert server.poll()is None and time.monotonic()<deadline;time.sleep(.05)
    ready=read(out/'ready.json');assert ready['jar_sha256'].upper()==p['jar_sha256'];endpoint='127.0.0.1:'+str(ready['port']);product=p['products'][a.profile]['a'];assert sha(product['path'])==product['sha256']
    launch([product['path'],'--native-login',endpoint,str(R.parent.parent/'0-客戶端')],'a')
    state.update(status='normal_windows_running_for_Sky_inputs',ready=ready,endpoint=endpoint,normal_products=p['products'][a.profile],runtime_deadline_seconds=240,expected_formal_sessions=2);persist()
    print(json.dumps(dict(output=str(out),endpoint=endpoint,normal_exe=product['path']),ensure_ascii=False),flush=True);deadline=time.monotonic()+270
    while any(child.poll()is None for child,*_ in children):assert time.monotonic()<deadline,'Focus check fixture budget expired';time.sleep(.1)
    assert all(child.returncode==0 for child,*_ in children)
    result=read(out/'result.json');assert result['actual_client_sessions']==2 and result['memberships_empty']and not result['sql_access']and result['authority_mutations']==0
    state.update(status='runtime_exited_original_zero_independent_GUI_qualification_pending',server_result=result)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    for child,row,stdout,stderr in children:
        if child.poll()is None:assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc'];child.kill();child.wait();row['terminated_during_failure_cleanup']=True
        row['exit_code']=child.returncode;stdout.close();stderr.close()
    state['recorded_owned_processes_terminal']=all(child.poll()is not None for child,*_ in children);persist()
print(json.dumps(dict(output=str(out),status=state['status'],whole_GUI_qualified=False),ensure_ascii=False))
