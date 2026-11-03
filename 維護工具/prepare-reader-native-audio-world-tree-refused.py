"""Build independent refusal reader from shared input/process verification only."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
s=(ROOT/'維護工具/qualify-native-audio-world-tree.py').read_text(encoding='utf-8').replace('native-audio-world-tree','native-audio-world-tree-refused')
prefix=s.split('pins={};packets={};pictures={};owners={}')[0]
suffix=s[s.index("k=ctypes.WinDLL('kernel32'"):]
middle=r'''
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
'''
suffix=suffix.replace('independently_qualified_same_connection_arena_WORLD_audio_and_authority','independently_qualified_world_tree_refusal_retains_arena_audio_and_authority')
suffix=suffix.replace('original_session_id=a.session','original_session_id=cli.session').replace('terminal_tool_chunk=a.terminal_chunk','terminal_tool_chunk=cli.terminal_chunk')
(ROOT/'維護工具/qualify-native-audio-world-tree-refused.py').write_text(prefix+'cli=a\n'+middle+suffix,encoding='utf-8',newline='\n')
print('Independent refusal reader prepared.')
