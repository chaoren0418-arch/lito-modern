"""Read actual ability requests, broadcasts, MP/score packets, passive authority and PNGs."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
from PIL import Image
import argparse,ctypes,hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int);ap.add_argument('--terminal-chunk');ap.add_argument('--receipt-observation-chunk');a=ap.parse_args()
if a.receipt_observation_chunk:
    assert a.session is None and a.terminal_chunk is None and a.receipt_observation_chunk not in ['pending','unknown']
else:assert a.session and a.session>0 and a.terminal_chunk not in [None,'','pending','unknown']
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native44FE-ability-buff-requests');c=read(run/'receipt.json')
assert c['status']=='executed_current75_ability_live_pending_independent_qualification'and c['owned_processes']==[]and c['asset_metadata_unchanged']
assert c['actual_sessions']==4 and c['actual_cast_requests']==4 and c['successful_casts']==2 and c['MP_rejections']==1 and c['source_overlays_used']and not c['normal_product_modified']
build=Path(c['compile']);b=read(build/'receipt.json');assert b['ability_request_candidate_included']and sha(build/'receipt.json')==c['compile_receipt_sha256']
assert c['candidate_manifest_sha256']==b['candidate_manifest_sha256']==sha(build/'candidate-inputs.json')
assert c['source_manifest_sha256']==b['base_source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for n,h in read(build/'candidate-inputs.json')['candidate_files'].items():assert sha(build/n)==h,n
for p in b['products'].values():assert sha(p['path'])==p['sha256']
assert sha(c['reference_jar'])==c['reference_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(c['catalog'])==c['catalog_sha256']and sha(c['resource_qualification'])==c['resource_qualification_sha256']
rq=read(c['resource_qualification']);assert rq['candidate_manifest_sha256']==c['candidate_manifest_sha256']and rq['ability_packet_component_passed']==5 and rq['ability_world_cases_passed']==2
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
for n,h in c['harness_class_pins'].items():assert sha(n)==h,n
work=Path(c['runtime']);server=read(work/'server/result.json');assert server['actual_client_sessions']==4 and server['game_sessions']==2 and server['memberships_empty']and server['authority_mutations']==0 and not server['sql_access']
assert server['profile']=='melee'and server['initial_fixture']=='default memory characters'
assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',(run/'client.stdout').read_text('utf-8-sig'))==[('1','0','0','0','0')]
frames=[]
for p in (work/'server/captures').rglob('frames.jsonl'):frames.extend(json.loads(x)for x in p.read_text('utf-8-sig').splitlines())
requests=[bytes.fromhex(r['plain'])for r in frames if r['direction']=='client_to_server'and r['opcode']=='0x14']
assert requests==[bytes.fromhex(s)for s in ['14030101000000','14030103000000','14050103000000','14030103000000']],requests
effects=[bytes.fromhex(r['plain'])for r in frames if r['direction']=='server_to_client'and r['opcode']=='0x37']
relevant=[p for p in effects if len(p)==7 and int.from_bytes(p[5:7],'little')in [750,6342]]
assert sorted(relevant)==sorted([bytes([0x37])+int(3).to_bytes(4,'little')+gfx.to_bytes(2,'little')for gfx in [750,6342]for _ in range(2)])
actions=[bytes.fromhex(r['plain'])for r in frames if r['direction']=='server_to_client'and r['opcode']=='0x20']
assert actions.count(bytes.fromhex('200300000013'))==4,'two real cast actions, broadcast to two clients'
assert sum(r['plain']=='MASKED_C_LOGIN'for r in frames)==4
assert sum(r['direction']=='server_to_client'and int(r['opcode'],16)==0x10 and bytes.fromhex(r['plain'])==bytes.fromhex('10160100')for r in frames)==1,'one actual MP278 refusal'
bresult=read(work/'b-result.json');assert bresult['error']is None and len(bresult['actions'])==4
mp_packets=[bytes.fromhex(r['plain'])for r in frames if r['direction']=='server_to_client'and int(r['opcode'],16)==0x4d]
score_packets=[bytes.fromhex(r['plain'])for r in frames if r['direction']=='server_to_client'and int(r['opcode'],16)in [0x6b,0x6c]]
for index,row in enumerate(bresult['actions']):
    before=row['before'];after=row['after'];assert before==row['immediately_after_queue']and row['duplicate_error']and before['inventory']==after['inventory']
    assert row['index']==index and row['skill']==(42 if index==2 else 26)and row['target']==(1 if index==0 else 3)
    if index in [0,3]:
        assert row['feedback']==('NoResponse'if index==0 else{'Rejected':278})and before['world']['mp']==after['world']['mp']and before['world']['status']==after['world']['status']
        assert before['world']['ability_buffs']==after['world']['ability_buffs']
        assert before['world']['own_cast_packets']==after['world']['own_cast_packets']
    else:
        assert row['feedback']=='CastAction'and before['world']['mp'][0]-after['world']['mp'][0]==50
        assert before['world']['own_cast_packets']+1==after['world']['own_cast_packets']
        ability='dexterity'if index==1 else'strength';assert after['world']['status'][ability]==before['world']['status'][ability]+5
        expected_mp=bytes([0x4d])+b''.join(n.to_bytes(2,'little')for n in after['world']['mp']);assert expected_mp in mp_packets
        s=after['world']['status'];expected_scores=bytes([0x6c,0xb0,4,s['dexterity'],5]if index==1 else[0x6b,0xb0,4,s['strength'],s['weight_gauge'],5]);assert expected_scores in score_packets
trace=[json.loads(line)['state']for line in (work/'server/server-trace.jsonl').read_text('utf-8-sig').splitlines()]
caster=[r for r in trace if r['test_account']=='PeerFixture2'and r['object_id']==3 and r['phase']=='BATTLE'];assert caster
assert caster[0]['mp']==bresult['initial']['world']['mp'][0]and caster[-1]['mp']==bresult['final']['world']['mp'][0]and caster[0]['mp']-caster[-1]['mp']==100
assert caster[0]['session_identity']==caster[-1]['session_identity']and caster[0]['character']==caster[-1]['character']
walk=read(work/'recovery-walk-target.json');assert walk==bresult['recovery_walk']
assert [caster[-1]['x'],caster[-1]['y']]==walk['target']and bresult['final']['world']['local_corrections']==walk['local_corrections']
move_packets=[bytes.fromhex(r['plain'])for r in frames if r['direction']=='client_to_server'and int(r['opcode'],16)==0x0a]
assert move_packets==[bytes([0x0a])+b''.join(n.to_bytes(2,'little')for n in walk['origin'])+bytes([walk['heading']])]
assert read(work/'a-walk-observed')==dict(actor=3,position=walk['target'])
assert [bresult['final']['world']['actors']['3'][k]for k in ['x','y']]==walk['target']
pictures={}
for role in ['a','b']:
    result=read(work/(role+'-result.json'));assert result['error']is None and set(result['pictures'])=={'750','6342'}
    assert [result['final']['world']['actors']['3'][k]for k in ['x','y']]==walk['target']
    shut=read(work/(role+'-shutdown.json'));assert shut['controller_quiescent']and shut['renderer_quiescent']
    for gfx,picture in result['pictures'].items():
        p=Path(picture['png']).resolve(strict=True);assert p.parent.samefile(work) and p.name==f'{role}-effect-{gfx}.png'
        with Image.open(p)as im:assert im.size==(800,600)and im.mode=='RGBA';im.load()
        bank=picture['receipt']['skill_effects']['additional_bank'];assert bank['loaded']and bank['load_error']is None and bank['sources']
        pictures[role+'-'+gfx]=dict(path=str(p),sha256=sha(p))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.CloseHandle.argtypes=[wintypes.HANDLE]
identities=[c['runner']]+c['process_history'];assert len(identities)==4
for row in identities:
    if 'exit_code'in row:assert row['exit_code']==0 and not row.get('cleanup_stopped_original_owned_handle')
    handle=k.OpenProcess(0x100000|0x1000,False,row['pid'])
    if not handle:assert ctypes.get_last_error()==87;continue
    try:
        ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
        created=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
        assert created!=row['creation_utc']or k.WaitForSingleObject(handle,0)==0
    finally:k.CloseHandle(handle)
out=run/'qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files={f'run/{p.name}':p for p in run.iterdir()if p.is_file()};files.update({f'runtime/{p.relative_to(work).as_posix()}':p for p in work.rglob('*')if p.is_file()and 'classes'not in p.relative_to(work).parts})
pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'qualified.json',dict(status='isolated_current75_ability_self_and_nonclan_scope_qualified',profile=c['profile'],source_manifest_sha256=c['source_manifest_sha256'],candidate_manifest_sha256=c['candidate_manifest_sha256'],
    original_session=a.session,observed_terminal_chunk=a.terminal_chunk,observed_terminal_exit_code=0 if a.terminal_chunk else None,receipt_observation_chunk=a.receipt_observation_chunk,wrapper_terminal_tool_observed=bool(a.terminal_chunk),child_exit_codes_verified=[r['exit_code']for r in c['process_history']],actual_sessions=4,actual_cast_requests=4,successful_self_casts=2,nonclan_target_rejections=1,MP_rejections=1,
    actual_C14_hex=[p.hex()for p in requests],actual_S37_broadcasts=4,source_authority_MP_cost=100,server_score_increment_each=5,MP_refusal_recovery_walk=walk,original_PNGs=pictures,recorded_identities=4,recorded_original_owned_processes_terminal=True,
    raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),source_overlays_used=True,normal_product_modified=False,same_clan_positive_target_verified=False,natural_expiry_verified=False,buff_GUI_verified=False,normal_GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False))
print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=4,successful_self_casts=2,nonclan_rejections=1),ensure_ascii=False))
