"""Eight automated normal-EXE hotel cases after the current limited normal baseline."""
import argparse, os, subprocess
from datetime import datetime,timezone
from native_catalog_hotel_gate import *

ap=argparse.ArgumentParser(); ap.add_argument('--normal-cohort',type=Path,required=True); a=ap.parse_args()
queue=read(ROOT/'工作佇列.json'); assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
assert queue['current_product_identity']['source_manifest_sha256']==SOURCE
base,lc,normal,n,inputs=baseline(a.normal_cohort)
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'; catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(jar)==sha(formal)==JAR and sha(catalog)==CATALOG
names=['run-native-catalog-hotel.py','native_catalog_hotel_gate.py','qualify-native-catalog-hotel-rooms.py','qualify-native-catalog-hotel-keys.py','qualify-native-catalog-hotel-share.py','qualify-native-catalog-hotel.py','verify-native-real-peers.ps1','verify-real-peer-rental-entry.py','verify-real-peer-rental-hotel.py','verify-real-peer-rental-hotel-share.py','verify-real-peer-rental-hotel-entry.py','native_hotel_catalog_reference.py','native_resident_map_reference.py','verify-real-peer-attributes.py','owned-peer-validation-binding.ps1']
pins={name:sha(T/name) for name in names}; refs={str(p):sha(p) for p in [jar,formal,catalog]}
out=V/'native-catalog-hotel'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'); out.mkdir(parents=True)
for name in names:(out/name).write_bytes((T/name).read_bytes())
for name in ['source-inputs.json','compiled-inputs.json']:(out/name).write_bytes((normal/name).read_bytes())
state=dict(status='running_current_catalog_normal_hotel_eight_cases',source_manifest_sha256=SOURCE,source_files=1047,compiled_input_files=1048,normal_cohort=str(normal),normal_baseline=str(base),normal_baseline_qualification_sha256=sha(base/'qualified/qualified.json'),full_current_library_qualified=False,normal_mains={p:x['normal']['sha256'] for p,x in n['profiles'].items()},reference_jar_sha256=JAR,resource_profile='tw13081901-v1',extended_item_details=True,validation_pins=pins,reference_pins=refs,runner_identity=identity(),process_history=[],owned_process=None,cases={},normal_source_modified=False,launcher_artifact_modified=False,diagnostic_API_extension=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,natural_expiry_verified=False,manual_guest_exit_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state); print('CURRENT_CATALOG_HOTEL='+str(out),flush=True)
env=dict(os.environ,PYTHONIOENCODING='utf-8',PYTHONDONTWRITEBYTECODE='1',TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
for key in list(env):
    if key.startswith('LITO_'):del env[key]

def guard():
    bind(out,terminal=False)
    q=read(ROOT/'工作佇列.json'); active=q.get('active_workloads',{})
    assert set(active)<= {'G06_hotel'}
    if active:
        assert active['G06_hotel']['run']==str(out)
        assert q['active_workload_sessions'].get('G06_hotel')
        assert original_live([state['runner_identity']])[0]
    else:assert not q['source_and_corresponding_QA_frozen']

try:
    for profile in ['Debug','Release']:
        for action in ACTIONS:
            guard(); key=profile+'-'+action
            command=['pwsh','-NoProfile','-File',str(T/'verify-native-real-peers.ps1'),'-SkipBuild','-BuildProfile',profile,'-ReferenceJar',str(jar),'-ResourceProfile','tw13081901-v1','-CommonCatalogPath',str(catalog),'-ExtendedItemDetails','-ActionProfile',action]
            logpath=out/(key+'.log')
            with logpath.open('wb') as log:
                child=subprocess.Popen(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
                row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,key=key)
                state['process_history'].append(row); state['owned_process']=row; save(out/'cohort.json',state)
                try:row['exit_code']=child.wait()
                except BaseException:
                    if child.poll() is None:
                        assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc']
                        row['cleanup_stopped_original_owned_handle']=True; child.terminate(); child.wait(timeout=3)
                    row['exit_code']=child.returncode; raise
                finally:state['owned_process']=None; save(out/'cohort.json',state)
            raw=logpath.read_bytes()
            try:lines=raw.decode('utf-8-sig').splitlines(); encoding='utf-8'
            except UnicodeDecodeError:lines=raw.decode('cp950').splitlines(); encoding='cp950'
            matches=[x.removeprefix('PEER_CASE_OUTPUT=') for x in lines if x.startswith('PEER_CASE_OUTPUT=')]
            state['last_console_encoding']=encoding
            assert row['exit_code']==0,'Preserve actual runtime failure: '+key
            assert len(matches)==1,'Exact case output required'
            case=Path(matches[0]).resolve(strict=True); assert case.is_relative_to(V/'native-real-peers')
            state['cases'][key]=dict(directory=str(case),PowerShell_exit_code=row['exit_code'],receipt_sha256=sha(case/'receipt.json'),comparison_sha256=sha(case/ACTIONS[action][0]))
            save(out/'cohort.json',state); case_binding(state,profile,action); guard()
            print(key+' actual runtime and independent comparison passed',flush=True)
    state['status']='executed_current_catalog_normal_hotel_eight_cases_readback_pending'
except BaseException as error:state.update(status='failed_preserved',error=repr(error)); raise
finally:save(out/'cohort.json',state)
print(json.dumps(dict(cohort=str(out),actual_cases=len(state['cases']),independent_cohort_readback_pending=True),ensure_ascii=False))
