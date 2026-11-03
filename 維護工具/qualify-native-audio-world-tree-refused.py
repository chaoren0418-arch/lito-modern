"""Read raw C02 packets, owner audio events, inputs, PNGs and process identities."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-audio-world-tree-refused')
assert a.terminal_chunk not in {'pending','unknown',''},'actual observed terminal chunk required'
c=read(run/'receipt.json');assert c['status']=='executed_diagnostic_pending_independent_qualification' and c['actual_sessions']==4 and c['owned_processes']==[] and c['asset_metadata_unchanged']
runtime=Path(c['runtime']);build=Path(c['compile']);b=read(build/'receipt.json');inputs=read(build/'compiled-inputs.json')
assert sha(build/'receipt.json')==c['compile_receipt_sha256'] and sha(c['product'])==c['product_sha256']==b['product_sha256']
assert sha(c['reference_jar'])==c['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(c['catalog'])==c['catalog_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for row in b['libraries'].values():assert sha(row['path'])==row['sha256']
for n,h in c['harness_class_pins'].items():assert sha(n)==h,n
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')

cli=a
pins={};packets={};pictures={};owners={}
config=runtime/'source/server.properties';assert 'classic.world.compatibility.enabled=false' in config.read_text(encoding='utf-8-sig');pins[str(config)]=sha(config)
server=read(runtime/'source/result.json')
assert server['actual_client_sessions']==4 and server['game_sessions']==2 and server['memberships_empty'] and server['authority_mutations']==0 and not server['sql_access']
rows=[]
for p in (runtime/'source/captures').rglob('frames.jsonl'):
    pins[str(p)]=sha(p);rows.extend(json.loads(x) for x in p.read_text(encoding='utf-8-sig').splitlines())
outgoing=[bytes.fromhex(r['plain']) for r in rows if r['direction']=='client_to_server' and r['plain']!='MASKED_C_LOGIN']
incoming=[bytes.fromhex(r['plain']) for r in rows if r['direction']=='server_to_client']
heals=[bytes.fromhex('14 00 00')+own.to_bytes(4,'little') for own in [1,3]];tree=bytes.fromhex('14 10 02')
assert sorted(b for b in outgoing if b[0]==0x14)==sorted(heals*2+[tree]*4)
assert sum(r['plain']=='MASKED_C_LOGIN' for r in rows)==4
assert not any(b[0]==0x5a for b in incoming) and bytes([8]) not in outgoing
refused=[b for b in incoming if b[0]==0x10 and len(b)>=3 and int.from_bytes(b[1:3],'little')==276]
assert len(refused)==4
for own in [1,3]:
    expected=bytes([0x37])+own.to_bytes(4,'little')+int(11722).to_bytes(2,'little')
    assert sum(b==expected for b in incoming)==4
packets=dict(actual_heal1_requests=4,actual_world_tree_requests=4,refusals276=4,S_PORTAL=0,C08_ack=0,actual_heal11722_broadcasts=8)
for role,own in [('a',1),('b',3)]:
    p=runtime/(role+'-result.json');j=read(p);pins[str(p)]=sha(p)
    before=j['before'];after=j['after_refusal'];walk=j['after_walk']
    assert j['epoch']==j['source_audio']['held']['epoch'] and j['retained_cue_deliveries']==1
    assert before['world']['generation']==after['world']['generation']==walk['world']['generation'] and before['world']['map_id']==after['world']['map_id']==walk['world']['map_id']==564
    assert after['phase']==walk['phase']=='battle_pending' and after['world']['portal'] is None
    assert before['selected_character']==after['selected_character']==walk['selected_character'] and before['inventory']==after['inventory']==walk['inventory']
    assert before['joined_room']['id']==after['joined_room']['id']==walk['joined_room']['id'] and after['channel']==before['channel']
    assert len(j['refusals'])==2
    for attempt in j['refusals']:
        a=attempt['before'];b=attempt['after'];immediate=attempt['immediate']
        assert attempt['duplicate_rejected'] and attempt['request_hex']=='141002'
        assert b['skill_feedback']=={'Rejected':276} and b['world']['message_count']>a['world']['message_count']
        assert a['inventory']==b['inventory']==immediate['inventory']
        for field in ['hp','mp','own_cast_packets','generation','map_id','completed_portals']:
            assert a['world'][field]==b['world'][field]==immediate['world'][field],(role,field)
    cue=j['held_cue_outcomes'];assert len(cue)==1 and cue[0]['first'].startswith('Played(Started { sound_id: 109,') and cue[0]['duplicate']=='AlreadyConsumed'
    source=j['source_audio'];follow=j['followup_audio'];event=source['held']['row']['event']
    assert source['deferred'] and not follow['deferred'] and event['actor']==own and event['sound']==109
    assert source['own_deliveries_before']==source['own_deliveries_after'] and follow['own_deliveries_after']==follow['own_deliveries_before']+1
    for audio in [source,follow]:
        assert audio['request_hex']==(bytes.fromhex('14 00 00')+own.to_bytes(4,'little')).hex()
        a=audio['before'];b=audio['after_ack'];immediate=audio['immediately_after_queue']
        assert a['inventory']==immediate['inventory'] and a['world']['hp']==immediate['world']['hp'] and a['world']['mp']==immediate['world']['mp']
        assert b['world']['own_cast_packets']==a['world']['own_cast_packets']+1
        assert any(e['graphics']==11722 and e['target'][0]==own for e in b['world']['effects'])
    owner=walk['world']['actors'][str(own)];assert [owner['x'],owner['y']]==j['walk_target'] and walk['world']['local_corrections']==after['world']['local_corrections']
    assert follow['own_delivery_events'][-1]['epoch']==j['epoch'] and follow['own_delivery_events'][-1]['row']['event']['actor']==own
    shutdown=read(runtime/(role+'-shutdown.json'));assert shutdown['client_quiescent'] and shutdown['view_quiescent'] and shutdown['error'] is None and shutdown['close_error'] is None
    for label in ['source','retained']:
        p=Path(j[label+'_picture']['png'])
        with Image.open(p) as im:
            im.load();assert im.size==(800,600) and im.mode=='RGBA';pixels=hashlib.sha256(im.tobytes()).hexdigest().upper()
        pictures[str(p)]=dict(sha256=sha(p),rgba_sha256=pixels)
    owners[role]=dict(held_event=event,first_submit=cue[0]['first'],repeat='AlreadyConsumed',retained_cue_deliveries=1,fresh_own_cue_deliveries=1,explicit_refused_requests=2)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.CloseHandle.argtypes=[wintypes.HANDLE];k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
identities=[]
for j in [read(run/'runner-identity.json')]+c['process_history']:
    if 'exit_code' in j:assert j['exit_code']==0 and not j.get('cleanup_stopped_original_owned_handle')
    h=k.OpenProcess(0x100000|0x1000,False,j['pid'])
    if not h:assert ctypes.get_last_error()==87;observation='PID absent'
    else:
        try:
            ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
            born=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
            reused=abs((born-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
            assert reused or k.WaitForSingleObject(h,0)==0;observation='PID reused' if reused else 'original signalled'
        finally:k.CloseHandle(h)
    identities.append(dict(pid=j['pid'],creation_utc=j['creation_utc'],original_terminal=True,observation=observation))
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'reader.py').write_bytes(Path(__file__).read_bytes())
result=dict(status='independently_qualified_world_tree_refusal_retains_arena_audio_and_authority',run=str(run),original_session_id=cli.session,original_exit_code=0,terminal_tool_chunk=cli.terminal_chunk,profile=c['profile'],source_manifest_sha256=c['source_manifest_sha256'],product_sha256=c['product_sha256'],actual_sessions=4,packets=packets,owner_audio=owners,pictures=pictures,raw_pins=pins,recorded_identities=identities,GUI_verified=False,physical_audio_verified=False,C72_audio_handoff_verified=False,authority_continuity_verified=True,whole_G06_complete=False,whole_goal_complete=False)
(out/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=4,owners=2,profile=c['profile']),ensure_ascii=False))
