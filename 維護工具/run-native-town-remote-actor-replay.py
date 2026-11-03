"""Compare two offline town fixtures with existing immutable diagnostic PEs."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import copy,ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
BASE=ROOT/'驗證紀錄/current/native-G06-controller/20261017-034558-407944'
OLD=ROOT/'驗證紀錄/current/native-G06-controller/20261017-030505-804788/town-load-replay/20261017-033321-769292'
CASE=ROOT/'驗證紀錄/current/native-teleport-failure/20261017-035002-519'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def created(p):
    k=ctypes.WinDLL('kernel32',use_last_error=True);t=[wintypes.FILETIME() for _ in range(4)]
    k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
    assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(v) for v in t])
    ticks=t[0].dwLowDateTime|t[0].dwHighDateTime<<32
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
old=read(OLD/'cohort.json');f=read(OLD/'fixture.json');assert old['owned_process'] is None
case=read(CASE/'receipt.json');capture=next((Path(case['runtime'])/'captures').glob('*/frames.jsonl'))
rows=[json.loads(s) for s in capture.read_text(encoding='utf-8-sig').splitlines()]
first_move=next(i for i,x in enumerate(rows) if x['direction']=='client_to_server' and x['plain'].startswith('0A '))
expected=read(CASE/'a-failure.json')['before']['world']['actors']['3']
# Own-spawn uses HP 255 while the captured remote snapshot has HP 100.
# Select the unique payload matching that recorded state, then let World.accept
# and the immutable replay's field assertions independently verify its bytes.
remote=[(i+1,x) for i,x in enumerate(rows[:first_move]) if x['direction']=='server_to_client' and x['note'].startswith('S_PUT_OBJECT object=3 ') and ('hp-ratio='+str(expected['hp_ratio'])+' ') in x['note']]
assert remote and len({x['plain'] for _,x in remote})==1
line,packet=remote[-1]
baseline=copy.deepcopy(f['worlds']['a']);assert baseline['expected_actor_count']==14 and '3' not in baseline['expected_actor_fields']
plus=copy.deepcopy(baseline);plus['expected_actor_count']=15
plus['packets'].append(dict(capture_LF_line=line,payload_hex=packet['plain'].replace(' ', '').lower(),note=packet['note']))
plus['expected_actor_fields']['3']={k:expected[k] for k in ['id','x','y','graphics','walk_action','heading','speed','brave','hp_ratio','status','name','lawful','level']}
fixture=dict(scope='Offline composed comparison: a=14 actors, b=same world plus one actual captured stationary remote Mage. Labels a/b are fixture variants, not live players.',
    source_manifest_sha256=old['source_manifest_sha256'],baseline_fixture=str(OLD/'fixture.json'),baseline_fixture_sha256=sha(OLD/'fixture.json'),
    additional_capture=str(capture),additional_capture_sha256=sha(capture),additional_capture_LF_line=line,
    worlds=dict(a=baseline,b=plus),actual_new_sessions=0)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=BASE/('town-remote-actor-replay-'+stamp);out.mkdir()
runtime=PROJECT/'target/current/native-town-remote-actor-replay'/stamp;runtime.mkdir(parents=True)
save(out/'fixture.json',fixture);(out/'runner.py').write_bytes(Path(__file__).read_bytes())
state=dict(status='running_offline_14_versus_15_actor_town_replay',output=str(out),runtime=str(runtime),source_manifest_sha256=old['source_manifest_sha256'],
    reused_diagnostic_compilation=str(OLD),profiles={},process_history=[],owned_process=None,actual_new_sessions=0,product_sources_changed=False,
    OS_cache_not_flushed=True,other_host_load_not_excluded=True,performance_accepted=False,GUI_verified=False,whole_goal_complete=False)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
save(out/'cohort.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    for profile in ['Debug','Release']:
        info=old['profiles'][profile];assert sha(info['executable'])==info['executable_sha256']
        d=runtime/profile;d.mkdir();(d/'fixture.json').write_bytes((out/'fixture.json').read_bytes())
        exe=d/'immutable-town-load-replay.exe';exe.write_bytes(Path(info['executable']).read_bytes());assert sha(exe)==info['executable_sha256']
        folder=out/profile;folder.mkdir();args=[str(exe),r'D:\lineage開發\0-客戶端',str(d)]
        with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
            p=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            item=dict(pid=p.pid,creation_utc=created(p),arguments=args);state['process_history'].append(item);state['owned_process']=item
            save(out/'cohort.json',state);code=p.wait();item['exit_code']=code;state['owned_process']=None;save(out/'cohort.json',state)
        assert code==0,'Preserve replay failure before changing any input'
        result=read(d/'results.json');assert [v['actors'] for v in result['records']]==[14,15]
        (folder/'results.json').write_bytes((d/'results.json').read_bytes())
        state['profiles'][profile]=dict(executable=str(exe),executable_sha256=sha(exe),results_sha256=sha(folder/'results.json'),
            summary=[{k:v[k] for k in ['role','actors','load_us','first_draw_us']} for v in result['records']])
        save(out/'cohort.json',state);print(json.dumps(dict(profile=profile,summary=state['profiles'][profile]['summary'])),flush=True)
    state['status']='executed_offline_14_versus_15_actor_town_replay'
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'cohort.json',state)
