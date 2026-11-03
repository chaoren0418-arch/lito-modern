"""Exact-source prerequisites and shared readback for current normal hotel cases."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import ctypes, hashlib, json, os, subprocess, time

ROOT=Path(__file__).resolve().parents[1]; P=ROOT/'LiTo381-Modern'; V=ROOT/'驗證紀錄/current'; T=ROOT/'維護工具'
SOURCE='33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21'
JAR='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
CATALOG='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
ACTIONS={'rental-hotel-entry':('rental-hotel-entry-comparison.json','verified_hotel_room_entry'),'rental-hotel-middle-entry':('rental-hotel-entry-comparison.json','verified_hotel_room_entry'),'rental-hotel':('rental-hotel-comparison.json','verified_hotel_key_request_and_return'),'rental-hotel-share':('rental-hotel-share-comparison.json','verified_shared_key_transfer_and_owner_return')}

def read(path):
    for i in range(10):
        try:return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)

def sha(path):
    with Path(path).open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()

def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')

k=ctypes.WinDLL('kernel32',use_last_error=True); k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    rows=[wintypes.FILETIME() for _ in range(4)]
    assert k.GetProcessTimes(handle,*[ctypes.byref(row) for row in rows])
    ticks=rows[0].dwLowDateTime|(rows[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()

def identity():return dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess()))

def original_live(rows):
    assert rows and all(isinstance(x['pid'],int) and x['pid']>0 for x in rows)
    filters=' OR '.join('ProcessId='+str(pid) for pid in sorted({x['pid'] for x in rows}))
    command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
    result=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert result.returncode==0 and not result.stderr
    actual=json.loads(result.stdout.decode('utf-8-sig')) if result.stdout.strip() else []
    if isinstance(actual,dict):actual=[actual]
    live=[row for row in rows if any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'])-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)]
    return live,result.stdout

def library(path):
    path=Path(path).resolve(strict=True); assert path.is_relative_to(V/'native33A3-library-default')
    c=read(path/'cohort.json'); o=read(path/'completion-observation.json')
    assert c['status']=='executed_both_current_full_library_default_selections' and c['owned_process'] is None
    assert o['status']=='independently_qualified_current1047_library_both_profiles_and_terminal_processes'
    assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal'] and o['cohort_sha256']==sha(path/'cohort.json')
    assert c['source_manifest_sha256']==o['source_manifest_sha256']==SOURCE
    normal=Path(c['normal_qualification']).parent.parent; n=read(normal/'cohort.json')
    assert n['status']=='executed_ability_finite_normals_related' and n['source_manifest_sha256']==SOURCE
    inputs=read(normal/'compiled-inputs.json'); assert len(inputs)==1048 and len(read(normal/'source-inputs.json'))==1047
    for profile in ['Debug','Release']:
        q=read(path/profile/'full-default-qualified/qualified.json')
        assert q['status']=='independently_qualified_current_full_library_default_selection' and q['profile']==profile and q['source_manifest_sha256']==SOURCE and q['failed']==0 and q['recorded_profile_owned_processes_terminal']
        candidates=list(normal.glob(profile+'-qualified-*/qualified.json')); assert len(candidates)==1
        nq=read(candidates[0]); assert nq['status']=='independently_qualified_ability_finite_normal_related_scope' and nq['profile']==profile and nq['source_manifest_sha256']==SOURCE
        assert nq['passed']==31 and nq['failed']==0 and nq['recorded_owned_all_terminal'] and not nq['source_overlays_used']
        assert sha(n['profiles'][profile]['normal']['path'])==nq['normal']['sha256']==n['profiles'][profile]['normal']['sha256']
    for name,digest in inputs.items():assert sha(ROOT/name)==digest,name
    return path,c,normal,n,inputs

def bind(run,terminal=True):
    run=Path(run).resolve(strict=True); assert run.is_relative_to(V/'native33A3-hotel')
    g=read(run/'cohort.json'); _,_,normal,n,inputs=library(g['library_cohort'])
    assert sha(Path(g['library_cohort'])/'completion-observation.json')==g['library_completion_sha256']
    assert g['source_manifest_sha256']==SOURCE and g['normal_cohort']==str(normal) and read(run/'compiled-inputs.json')==inputs
    assert read(run/'source-inputs.json')==read(normal/'source-inputs.json')
    assert sha(run/'source-inputs.json')==SOURCE
    assert g['normal_mains']=={p:x['normal']['sha256'] for p,x in n['profiles'].items()}
    for name,digest in g['validation_pins'].items():assert sha(T/name)==sha(run/name)==digest,name
    for name,digest in g['reference_pins'].items():assert sha(name)==digest,name
    assert g['reference_jar_sha256']==JAR and g['resource_profile']=='tw13081901-v1' and g['extended_item_details']
    assert not g['normal_source_modified'] and not g['launcher_artifact_modified'] and not g['diagnostic_API_extension']
    if terminal:
        assert g['status']=='executed_current33A3_normal_hotel_eight_cases_readback_pending' and g['owned_process'] is None
        assert set(g['cases'])=={p+'-'+a for p in ['Debug','Release'] for a in ACTIONS}
        assert len(g['process_history'])==8 and all(row['exit_code']==0 and not row.get('cleanup_stopped_original_owned_handle') for row in g['process_history'])
    return g,normal,n,inputs

def case_binding(g,profile,action):
    binding=g['cases'][profile+'-'+action]; assert binding['PowerShell_exit_code']==0
    case=Path(binding['directory']).resolve(strict=True); assert case.is_relative_to(V/'native-real-peers')
    r=read(case/'receipt.json'); runtime=Path(r['runtime']).resolve(strict=True)
    assert runtime.is_relative_to(P/'target/current/native-real-peers')
    comparison,status=ACTIONS[action]
    assert sha(case/'receipt.json')==binding['receipt_sha256'] and sha(case/comparison)==binding['comparison_sha256']
    assert read(case/comparison)['status']==status
    assert r['exe_sha256']==g['normal_mains'][profile]==sha(runtime/'qualified-client.exe')
    assert r['jar_sha256']==JAR and r['source_matches_snapshot'] and r['action_profile']==action and r['resource_profile']=='tw13081901-v1' and r['extended_item_details']
    sourcepins=read(case/'source-pins.json')
    for name,digest in sourcepins.items():assert sha(P/name)==digest,name
    server=r['server']; assert server['actual_client_sessions']==4 and server['game_sessions']==2 and server['memberships_empty']
    assert server['authority_mutations']==server['classic_world_members_after_disconnect']==server['active_hotel_leases_after_disconnect']==0
    assert not server['sql_access'] and not server['state_persistent']
    marker=read(case/'validation-completed.json'); assert marker['receipt_sha256']==sha(case/'receipt.json')
    assert set(marker['comparison_pins'])=={'rental-entry-comparison.json',comparison,'attributes-comparison.json'}
    for name,digest in marker['comparison_pins'].items():assert sha(case/name)==digest and read(case/name)['receipt_sha256'].upper()==sha(case/'receipt.json')
    for role in ['server','a','b']:assert read(case/(role+'-launch-identity.json'))['exe_sha256']==g['normal_mains'][profile]
    return case,r,runtime
