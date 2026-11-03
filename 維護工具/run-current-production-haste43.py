"""Current public production-rlib APIs against unmodified formal75; no Runtime overlays."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current';T=R/'維護工具'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--scenario',choices=['self-peer','mp-refused','cancel-repaired'],default='self-peer');args=ap.parse_args()
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
source=w['current_product_identity']['source_manifest_sha256'];assert source=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
from text_index_order_runtime_gate import require_idle_runtime
require_idle_runtime(R)
if args.scenario!='self-peer':
    positive=w['current_native_haste43_production_self_peer_formal75'];assert positive['source_manifest_sha256']==source and positive['qualified'] and positive['actual_formal_casts']==4 and sha(positive['qualification'])==positive['qualification_sha256']
normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run']);inputs=read(normal/'compiled-inputs.json')
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371' and sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
extra_pins={}
normal_q=read(w['current_native_installed_resource_profile_normal_qualification']['qualification'])
def guard():
    assert all(sha(R/n)==h for n,h in inputs.items())
    assert all(sha(p)==h for p,h in extra_pins.items())
    assert all(sha(info['normal']['path'])==normal_q['normal_mains'][profile] for profile,info in normal_q['profiles'].items())
guard()
out=V/'native-haste43-live-current'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
body=T/'native-haste43-production-body.rs';(out/'live-body.rs').write_bytes(body.read_bytes())
(out/'main.rs').write_text('include!(r#"'+str(out/'live-body.rs')+'"#);\n',encoding='utf-8',newline='\n')
fixture=P/'tests/oracle/NativePeerHarness.java';(out/'NativePeerHarness.java').write_bytes(fixture.read_bytes())
extra_pins.update({str(p):sha(p) for p in [out/'live-body.rs',out/'main.rs',out/'NativePeerHarness.java',out/'runner.py',body]})
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(x) for x in ts]);ticks=ts[0].dwLowDateTime|ts[0].dwHighDateTime<<32
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='preparing_current_production_haste43',source_manifest_sha256=source,pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_processes=[],profiles={},compiled_inputs=inputs,generated_source_pins=dict(extra_pins),fixture_origin=str(fixture),fixture_sha256=sha(fixture),jar=str(jar),jar_sha256=sha(jar),catalog=str(catalog),catalog_sha256=sha(catalog),normal_qualification=w['current_native_installed_resource_profile_normal_qualification']['qualification'],normal_qualification_sha256=w['current_native_installed_resource_profile_normal_qualification']['qualification_sha256'],normal_product_modified=False,production_rlib_used=True,production_source_overlays_used=False,candidate_applied=False,GUI_verified=False,whole_goal_complete=False)
state['scenario']=args.scenario;state['fixture_profile']='heal19-refused' if args.scenario=='mp-refused' else 'melee'
save(out/'cohort.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
env={n:v for n,v in os.environ.items() if not n.startswith('LITO_')};env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'),CARGO_MANIFEST_DIR=str(P))
owned=[]
def launch(command,folder,role):
    guard();folder.mkdir(parents=True,exist_ok=True);stdout=(folder/(role+'.stdout.log')).open('wb');stderr=(folder/(role+'.stderr.log')).open('wb')
    child=subprocess.Popen(command,cwd=folder,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    row=dict(role=role,pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,exit_code=None)
    state['process_history'].append(row);owned.append((child,row,stdout,stderr));state['owned_processes']=[x[1] for x in owned if x[0].poll() is None];save(out/'cohort.json',state);return child,row
def finish(child,row,seconds,expected=0):
    try:child.wait(timeout=seconds)
    except subprocess.TimeoutExpired:
        assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc'];child.kill();child.wait(timeout=10);row['exact_owned_timeout_cleanup']=True;raise
    finally:
        row.update(exit_code=child.returncode,finished_utc=datetime.now(timezone.utc).isoformat());state['owned_processes']=[x[1] for x in owned if x[0].poll() is None];save(out/'cohort.json',state)
    assert child.returncode==expected,(row['role'],child.returncode,expected)
    guard()
jdk=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin');rustc=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rustc.exe');target='x86_64-pc-windows-msvc'
try:
    classes=out/'classes';classes.mkdir();child,row=launch([str(jdk/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/'NativePeerHarness.java')],out,'javac');finish(child,row,60)
    for profile in ['Debug','Release']:
        build=normal/profile/'test-build/stdout.log';messages=[json.loads(l) for l in build.read_text('utf-8-sig').splitlines() if l.startswith('{')]
        deps=P/'target/current'/target/profile.lower()/'deps';extern={}
        for name in ['lito_modern','anyhow','serde','serde_json','png','zeroize','windows']:
            paths={str(Path(f).resolve(strict=True)) for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']==name for f in m['filenames'] if f.endswith('.rlib') and Path(f).parent==deps};assert len(paths)==1,(profile,name,paths);extern[name]=paths.pop()
        extra_pins.update({p:sha(p) for p in extern.values()})
        work=out/profile;work.mkdir();exe=work/'haste43-live.exe';command=[str(rustc),str(out/'main.rs'),'--edition=2021','--target',target,'--crate-name','current_production_haste43_'+profile.lower(),'-o',str(exe),'-L','dependency='+str(deps),'-L','dependency='+str(P/'target/current'/profile.lower()/'deps')]
        command+=['-C','opt-level=0','-C','debuginfo=2'] if profile=='Debug' else ['-C','opt-level=z','-C','lto=thin','-C','codegen-units=16','-C','strip=symbols']
        for name,path in extern.items():command+=['--extern',name+'='+path]
        for path in sorted({p for m in messages if m.get('reason')=='build-script-executed' for p in m['linked_paths']}):command+=['-L',path]
        state.update(status='compiling_current_production_haste43',active_profile=profile);save(out/'cohort.json',state)
        child,row=launch(command,work,'rustc');finish(child,row,300)
        state['profiles'][profile]=dict(executable=str(exe),executable_sha256=sha(exe),extern={n:dict(path=p,sha256=sha(p)) for n,p in extern.items()})
        case=work/'case';case.mkdir();server,srow=launch([str(jdk/'java.exe'),'-Dfile.encoding=UTF-8','-Xverify:all','-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',str(classes)+';'+str(jar),'local.lito381.server.NativePeerHarness',str(case),state['fixture_profile']],case,'server')
        end=time.monotonic()+8
        while not(case/'ready.json').exists():assert server.poll() is None and time.monotonic()<end;time.sleep(.05)
        ready=read(case/'ready.json');assert ready['jar_sha256'].upper()==sha(jar)
        state.update(status='executing_current_production_haste43');save(out/'cohort.json',state)
        client,crow=launch([str(exe),'127.0.0.1:'+str(ready['port']),str(case),args.scenario],case,'client');finish(client,crow,140,1 if args.scenario=='cancel-retirement' else 0);finish(server,srow,15)
        proof=read(case/'result.json');assert proof['actual_client_sessions']==4 and proof['game_sessions']==2 and proof['memberships_empty'] and not proof['sql_access'] and not proof['state_persistent']
        cp=read(case/'client-result.json')
        if args.scenario=='self-peer':assert cp['actual_casts']==2 and cp['status']=='current_haste43_two_formal_casts_observed'
        elif args.scenario=='mp-refused':assert cp['actual_casts']==0 and cp['actual_requests']==1 and cp['status']=='current_haste43_low_mp_no_response_observed'
        else:
            assert cp['actual_casts']==1 and cp['finite755_retired_while_speed1']
            if args.scenario=='cancel-retirement':assert read(case/'error.json')['error']=='cancel retained previous world/room/skills after owned transport quiescence'
            else:assert not(case/'error.json').exists()
        # The client and original fixture use separate result files.
        state['profiles'][profile].update(server_result=proof,client_result=cp,case=str(case));save(out/'cohort.json',state);print(json.dumps(dict(profile=profile,status='original_formal_sessions_terminal'),ensure_ascii=False),flush=True)
    state.update(status='executed_current_production_haste43_both_profiles',cancel_counterexample_expected=args.scenario=='cancel-retirement')
except BaseException as e:
    state.update(status='failed_preserved',error=repr(e));raise
finally:
    for child,row,stdout,stderr in owned:
        if child.poll() is None:
            assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc'];child.kill();child.wait(timeout=10);row['terminated_during_failure_cleanup']=True
        row['exit_code']=child.returncode;stdout.close();stderr.close()
    state.update(owned_processes=[],finished_utc=datetime.now(timezone.utc).isoformat(),recorded_children_terminal=all(x[0].poll() is not None for x in owned));guard();save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],normal_product_modified=False),ensure_ascii=False),flush=True)
