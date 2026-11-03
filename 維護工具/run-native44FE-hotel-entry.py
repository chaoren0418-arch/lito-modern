"""Exercise current normal EXEs on real isolated hotel entry branches, without a build."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, subprocess, time

ROOT=Path(__file__).resolve().parents[1]; P=ROOT/'LiTo381-Modern'; V=ROOT/'驗證紀錄/current'; T=ROOT/'維護工具'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(h):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(x)for x in ts])
    ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']
assert queue['current_product_identity']['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
controller=V/'native1040-G06-controller/20261018-161827-160049';c=read(controller/'cohort.json')
assert read(controller/'completion-observation.json')['recorded_original_owned_processes_all_terminal']
inputs=read(controller/'source-inputs.json');assert len(inputs)==1040
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(jar)==sha(formal)==c['reference_jar_sha256']
assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
validation_names=['verify-native-real-peers.ps1','verify-real-peer-rental-entry.py','verify-real-peer-rental-hotel-entry.py','native_hotel_catalog_reference.py','native_resident_map_reference.py','verify-real-peer-attributes.py','owned-peer-validation-binding.ps1']
pins={str(T/n):sha(T/n)for n in validation_names};pins.update({str(x):sha(x)for x in [jar,formal,catalog]})
normal={profile:P/('target/current/x86_64-pc-windows-msvc/'+profile.lower()+'/LiToModern.exe')for profile in ['Debug','Release']}
def guard():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in pins.items():assert sha(n)==h,n
    for profile,path in normal.items():assert sha(path)==c['normal_mains'][profile],profile
guard()
ap=argparse.ArgumentParser();ap.add_argument('--resume',type=Path);a=ap.parse_args()
if a.resume:
    out=a.resume.resolve(strict=True);assert out.is_relative_to(V/'native44FE-hotel')
    state=read(out/'cohort.json');assert state['status']=='failed_preserved' and state['owned_process'] is None
    assert len(state['process_history'])==1 and state['process_history'][0]['exit_code']==0 and not state['cases']
    assert 'UnicodeDecodeError' in state['error'] and state['validation_pins']==pins
    # The actual first runtime succeeded. Preserve the failed reader and bind it without a rerun.
    (out/'initial-reader-failure.json').write_bytes((out/'cohort.json').read_bytes())
    lines=(out/'Debug-rental-hotel-entry.log').read_bytes().decode('cp950').splitlines()
    paths=[x.removeprefix('PEER_CASE_OUTPUT=')for x in lines if x.startswith('PEER_CASE_OUTPUT=')];assert len(paths)==1
    case=Path(paths[0]);r=read(case/'receipt.json');proof=read(case/'rental-hotel-entry-comparison.json')
    assert r['exe_sha256']==c['normal_mains']['Debug'] and r['jar_sha256']==c['reference_jar_sha256'] and r['action_profile']=='rental-hotel-entry'
    assert proof['status']=='verified_hotel_room_entry' and r['server']['actual_client_sessions']==4
    state['cases']['Debug-rental-hotel-entry']=dict(directory=str(case),PowerShell_exit_code=0,receipt_sha256=sha(case/'receipt.json'),comparison_sha256=sha(case/'rental-hotel-entry-comparison.json'),retained_without_rerun=True)
    state['previous_runner_identities']=[state['runner_identity']]
    state['initial_reader_failure_terminal']=dict(original_session_id=19083,exit_code=1,terminal_tool_chunk='6346af',runtime_PowerShell_exit_code=0,console_encoding='cp950',runtime_reexecuted=False)
    state['runner_identity']=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess()))
    state['status']='running_current_normal_hotel_room_entry';state.pop('error')
    (out/'resumed-runner.py').write_bytes(Path(__file__).read_bytes())
else:
    out=V/'native44FE-hotel'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir()
    (out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'source-inputs.json').write_bytes((controller/'source-inputs.json').read_bytes())
    state=dict(status='running_current_normal_hotel_room_entry',source_manifest_sha256=c['source_manifest_sha256'],reference_jar_sha256=c['reference_jar_sha256'],normal_mains=c['normal_mains'],
    runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],cases={},owned_process=None,validation_pins=pins,
    normal_source_modified=False,launcher_artifact_modified=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state)
print('HOTEL_ENTRY_COHORT='+str(out),flush=True)
env=dict(os.environ,PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1',TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
try:
    for profile in ['Debug','Release']:
        for action in ['rental-hotel-entry','rental-hotel-middle-entry']:
            guard();key=profile+'-'+action;console=out/(key+'.log')
            if key in state['cases']:
                retained=state['cases'][key];assert retained['retained_without_rerun'] and retained['PowerShell_exit_code']==0
                assert sha(Path(retained['directory'])/'receipt.json')==retained['receipt_sha256']
                assert sha(Path(retained['directory'])/'rental-hotel-entry-comparison.json')==retained['comparison_sha256']
                print(key+' retained without rerun',flush=True);continue
            command=['pwsh','-NoProfile','-File',str(T/'verify-native-real-peers.ps1'),'-SkipBuild','-BuildProfile',profile,'-ReferenceJar',str(jar),'-ResourceProfile','tw13081901-v1','-CommonCatalogPath',str(catalog),'-ExtendedItemDetails','-ActionProfile',action]
            with console.open('wb') as log:
                child=subprocess.Popen(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
                job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,key=key)
                state['process_history'].append(job);state['owned_process']=job;save(out/'cohort.json',state)
                job['exit_code']=child.wait();state['owned_process']=None
            matches=[x.removeprefix('PEER_CASE_OUTPUT=')for x in console.read_bytes().decode('cp950').splitlines()if x.startswith('PEER_CASE_OUTPUT=')]
            assert len(matches)==1
            case=Path(matches[0]);binding=dict(directory=str(case),PowerShell_exit_code=job['exit_code']);state['cases'][key]=binding;save(out/'cohort.json',state)
            assert job['exit_code']==0,'Runtime or independent comparison failed: '+key
            r=read(case/'receipt.json');assert r['exe_sha256']==c['normal_mains'][profile] and r['jar_sha256']==c['reference_jar_sha256'] and r['action_profile']==action
            assert r['server']['actual_client_sessions']==4 and r['server']['authority_mutations']==0 and r['server']['memberships_empty'] and r['server']['active_hotel_leases_after_disconnect']==0
            proof=read(case/'rental-hotel-entry-comparison.json');assert proof['status']=='verified_hotel_room_entry' and len(proof['checks'])==2
            guard();binding.update(receipt_sha256=sha(case/'receipt.json'),comparison_sha256=sha(case/'rental-hotel-entry-comparison.json'))
            save(out/'cohort.json',state);print(key+' executed and independent comparisons passed',flush=True)
    state['status']='executed_current_normal_hotel_room_entry_independent_cohort_readback_pending'
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'cohort.json',state)
print(json.dumps(dict(cohort=str(out),cases=len(state['cases']),qualification_pending=True),ensure_ascii=False))
