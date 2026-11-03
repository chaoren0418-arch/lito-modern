"""Aggregate only fresh normal-EXE hotel evidence after the original runner exits."""
import argparse,subprocess,sys,zipfile
from datetime import datetime,timezone
from native_catalog_hotel_gate import *

ap=argparse.ArgumentParser(); ap.add_argument('run',type=Path); ap.add_argument('--session',type=int,required=True); ap.add_argument('--terminal-chunk',required=True); ap.add_argument('--terminal-exit-code',type=int,required=True); a=ap.parse_args()
run=a.run.resolve(strict=True);g,normal,n,inputs=bind(run)
assert a.session>0 and a.terminal_exit_code==0 and a.terminal_chunk not in ['', 'unknown','pending']
identities=[g['runner_identity'],*g['process_history']]; sessions=0
for profile in ['Debug','Release']:
    for action in ACTIONS:
        case,r,runtime=case_binding(g,profile,action); sessions+=r['server']['actual_client_sessions']
        for role in ['server','a','b']:identities.append(read(case/(role+'-launch-identity.json')))
assert sessions==32 and len(identities)==33
live,raw=original_live(identities); assert not live
assert not (run/'qualified.json').exists()
(run/'process-CIM.json').write_bytes(raw)
env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
readers=[]
for mode in ['rooms','keys','share']:
    path=run/(mode+'-qualified/qualified.json')
    assert not path.exists(),'preserve any existing qualification or failed reader before a deliberate continuation'
    command=[sys.executable,'-B','-X','utf8',str(T/('qualify-native-catalog-hotel-'+mode+'.py')),str(run)]
    if mode=='rooms':command+=['--session',str(a.session),'--terminal-chunk',a.terminal_chunk]
    with (run/(mode+'-readback.log')).open('xb') as log:
        child=subprocess.Popen(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),mode=mode,command=command)
        readers.append(job);save(run/'readback-processes.json',readers)
        job['exit_code']=child.wait();save(run/'readback-processes.json',readers)
    assert job['exit_code']==0,'Preserve independent reader failure: '+mode
    bind(run)
qpaths={mode:run/(mode+'-qualified/qualified.json') for mode in ['rooms','keys','share']}
qs={mode:read(path) for mode,path in qpaths.items()}
assert qs['rooms']['actual_sessions']==16 and qs['rooms']['maps']==[501,502,503] and qs['rooms']['original_world_PNGs']==24
assert qs['keys']['actual_formal_sessions']==8 and qs['keys']['original_world_PNGs']==8
assert qs['share']['actual_formal_sessions']==8 and qs['share']['original_world_PNGs']==12
for mode,q in qs.items():
    assert q['source_manifest_sha256']==SOURCE and not q['GUI_verified'] and not q['whole_goal_complete']
    path=qpaths[mode].parent/('raw.zip' if mode=='rooms' else 'raw-evidence.zip')
    assert sha(path)==q['raw_archive_sha256']
    with zipfile.ZipFile(path) as z:
        hashes=q['raw_files'] if mode=='rooms' else q['raw_archive_files']
        assert set(z.namelist())==set(hashes)
        for name,digest in hashes.items():
            with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==digest
assert not original_live(identities+readers)[0]
bind(run)
q=dict(status='independently_qualified_current_catalog_normal_hotel_rooms_keys_and_shared_owner_return',cohort=str(run),cohort_sha256=sha(run/'cohort.json'),source_manifest_sha256=SOURCE,source_files=1047,compiled_input_files=1048,reference_jar_sha256=JAR,normal_mains=g['normal_mains'],normal_baseline=g['normal_baseline'],normal_baseline_qualification_sha256=g['normal_baseline_qualification_sha256'],full_current_library_qualified=False,original_session_id=a.session,observed_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk,actual_cases=8,actual_formal_sessions=32,LOGIN_sessions=16,GAME_sessions=16,maps=[501,502,503],original_world_PNGs=44,mode_qualifications={mode:dict(path=str(path),sha256=sha(path)) for mode,path in qpaths.items()},recorded_runtime_owned_identities=len(identities),recorded_original_owned_processes_all_terminal=True,recorded_runtime_identities=identities,readback_processes=readers,normal_source_modified=False,launcher_artifact_modified=False,diagnostic_API_extension=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,natural_expiry_verified=False,manual_guest_exit_verified=False,whole_G06_3d_complete=False,whole_goal_complete=False,qualified_utc=datetime.now(timezone.utc).isoformat())
save(run/'qualified.json',q);(run/'independent-cohort-qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(qualification=str(run/'qualified.json'),cases=8,sessions=32,PNGs=44,runtime_identities=33),ensure_ascii=False))
